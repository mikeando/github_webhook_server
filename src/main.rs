use anyhow::{Context, Result};
use async_trait::async_trait;
use git::CommandOutput;
use git::GitRepositoryError;
use serde::Deserialize;
use tide::Endpoint;
use tide::Request;
use tide::Response;
use tide::StatusCode;

use std::collections::BTreeMap;
use std::ffi::OsStr;

use std::sync::Mutex;
use std::sync::mpsc;

use crate::git::GitRepository;
use crate::github::GithubPushEvent;

pub mod git;
pub mod github;

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
struct HookConfig {
    name: String,
    repo_name: String,
    hook_route: String,
    repository_directory: String,
    script: String,
    branch: String,
    secret: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct Config {
    hooks: Vec<HookConfig>,
}

#[derive(Debug)]
pub struct Route {
    route: String,
    hooks: Vec<HookConfig>,
    channel: Mutex<mpsc::Sender<Event>>,
}

impl Route {
    pub fn new(route: String, channel: mpsc::Sender<Event>) -> Self {
        Route {
            route,
            hooks: vec![],
            channel: Mutex::new(channel),
        }
    }

    fn add_hook(&mut self, hook: HookConfig) {
        assert!(hook.hook_route == self.route);
        self.hooks.push(hook);
    }
}

#[derive(Debug)]
pub enum RouteError {
    TideError(tide::Error),
    GitRepositoryError(GitRepositoryError),
    DecodingError(serde_json::Error),
    AuthenticationError(String),
    ChannelError,
}

impl From<GitRepositoryError> for RouteError {
    fn from(e: GitRepositoryError) -> Self {
        RouteError::GitRepositoryError(e)
    }
}

#[derive(Default, Debug)]
struct SimpleLog {
    content: String,
}

impl SimpleLog {
    fn add_entry(&mut self, prefix: &str, s: &str) {
        use std::fmt::Write;
        let now = chrono::Local::now();
        let timestamp = now.to_rfc3339();
        for (i, l) in s.lines().enumerate() {
            if i == 0 {
                writeln!(self.content, "{}: {}:+:{}", timestamp, prefix, l).unwrap();
            } else {
                writeln!(self.content, "{}: {}:|:{}", timestamp, prefix, l).unwrap();
            }
        }
    }

    pub fn info<S: AsRef<str>>(&mut self, s: S) {
        self.add_entry("INFO", s.as_ref());
    }

    pub fn error<S: AsRef<str>>(&mut self, s: S) {
        self.add_entry("ERROR", s.as_ref());
    }
}

fn render_command_output_to_log(log: &mut SimpleLog, stage: &str, output: &CommandOutput) {
    log.info(output.format(stage));
}

fn render_command_error_to_log(log: &mut SimpleLog, stage: &str, err: &GitRepositoryError) {
    match err {
        GitRepositoryError::CommandError(e) => {
            log.error(format!("\n--------\n{}\n--------\n", stage));
            log.error(format!("error = {}", e));
        }
        GitRepositoryError::CommandFailed(output) => {
            log.error(output.format(stage));
        }
    }
}

fn render_log_to_stderr(log: &SimpleLog) {
    eprint!("{}", log.content);
}

fn handle_command_result(
    result: Result<CommandOutput, GitRepositoryError>,
    stage: &str,
    log: &mut SimpleLog,
) -> Result<(), GitRepositoryError> {
    match result {
        Ok(v) => {
            render_command_output_to_log(log, stage, &v);
            Ok(())
        }
        Err(v) => {
            render_command_error_to_log(log, stage, &v);
            eprintln!("Error {} - log follows", stage);
            render_log_to_stderr(log);
            Err(v)
        }
    }
}

fn handle_git_command<I, S>(
    args: I,
    stage: &str,
    log: &mut SimpleLog,
    repo: &GitRepository,
) -> Result<(), GitRepositoryError>
where
    I: IntoIterator<Item = S> + std::fmt::Debug,
    S: AsRef<OsStr>,
{
    handle_command_result(repo.run_git_command(args), stage, log)
}

fn handle_command(
    cmd: &str,
    stage: &str,
    log: &mut SimpleLog,
    repo: &GitRepository,
) -> Result<(), GitRepositoryError> {
    let no_args: &[&str] = &[];
    handle_command_result(repo.run_command(cmd, no_args), stage, log)
}

impl Route {
    pub fn route(&self) -> String {
        self.route.clone()
    }

    fn validate_signature(
        &self,
        hook: &HookConfig,
        req: &mut Request<()>,
        body: &[u8],
    ) -> Result<(), String> {
        if let Some(secret) = &hook.secret {
            // signature = 'sha256=' + OpenSSL::HMAC.hexdigest(OpenSSL::Digest.new('sha256'), ENV['SECRET_TOKEN'], payload_body)
            // return halt 500, "Signatures didn't match!" unless Rack::Utils.secure_compare(signature, request.env['HTTP_X_HUB_SIGNATURE_256'])

            let signature = req
                .header("X-Hub-Signature-256")
                .ok_or_else(|| "Missing X-Hub-Signature-256 header".to_string())?
                .last()
                .as_str();

            let signature = signature.strip_prefix("sha256=").ok_or_else(|| {
                format!(
                    "Malformed HTTP_X_HUB_SIGNATURE_256: should start with sha256= but was '{}'",
                    signature
                )
            })?;

            println!("Signature from headers is '{}'", signature);

            let signature_bytes = hex::decode(signature).map_err(|_| {
                format!(
                    "Malformed X-Hub-Signature-256: should be all hex, but was '{}'",
                    signature
                )
            })?;

            use ring::hmac;
            let key = hmac::Key::new(hmac::HMAC_SHA256, secret.as_bytes());

            let tag = hmac::sign(&key, body);
            println!(
                "signature from key and body is {}",
                hex::encode(tag.as_ref())
            );

            hmac::verify(&key, &body, &signature_bytes)
                .map_err(|_| "Invalid message signature".to_string())?;
        }
        Ok(())
    }

    fn hook_for_event(&self, v: &GithubPushEvent) -> Option<&HookConfig> {
        for hook in &self.hooks {
            if v.repository.full_name == hook.repo_name
                && v.reference.0 == format!("refs/heads/{}", hook.branch)
            {
                return Some(hook);
            }
        }
        None
    }

    pub async fn process_request(&self, req: &mut Request<()>) -> Result<(), RouteError> {
        // We cant use the body_json method directly as we need to get the raw bytes to check the
        // secret is correct. But we can't validate the body until we've built the object
        // since we dont know which hook it corresponds to.

        let body = match req.body_bytes().await {
            Ok(body) => body,
            Err(e) => {
                eprintln!("Error receiving webhook:\n{:?}\n", e);
                return Err(RouteError::TideError(e));
            }
        };

        let v = serde_json::from_slice(&body);
        let v: GithubPushEvent = match v {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Error decoding GithubPushEvent:\n{:?}\n", e);
                return Err(RouteError::DecodingError(e));
            }
        };
        eprintln!("Got GithubPushEvent:\n{:#?}\n", v);

        let hook: &HookConfig = match self.hook_for_event(&v) {
            Some(v) => v,
            None => {
                eprintln!("No valid hook found");
                return Err(RouteError::AuthenticationError(
                    "No valid hook found".into(),
                ));
            }
        };
        eprintln!("Using hook: {}", hook.name);

        if let Err(e) = self.validate_signature(hook, req, &body) {
            eprintln!("Error validating webhook:\n{:?}\n", e);
            return Err(RouteError::AuthenticationError(e));
        }

        self.channel.lock().map_err(|_e| RouteError::ChannelError)?
            .send(
                Event::PushEvent( PushEvent{
                    hook: hook.clone(),
                    content: v
                })
            ).map_err(|_e| RouteError::ChannelError)?;

        // TODO: Validate that this event is for the repository we care about
        //       and the branches we care about.



        Ok(())
    }
}

#[async_trait]
impl Endpoint<()> for Route {
    async fn call(&self, req: Request<()>) -> tide::Result {
        let mut req = req;
        match self.process_request(&mut req).await {
            Ok(_) => Ok("".into()),
            Err(e) => {
                eprintln!("Error processing request: {:?}", e);
                let mut res = Response::new(StatusCode::InternalServerError);
                res.set_body(format!("{:?}", e));
                Ok(res)
            }
        }
    }
}

pub struct PushEvent {
    hook: HookConfig,
    content: GithubPushEvent,
}

pub enum Event {
    Done,
    PushEvent(PushEvent),
}

fn update_and_run_hook(hook: &HookConfig) -> Result<(), RouteError> {
    let git = "git";
    let repo = GitRepository {
        repo_dir: hook.repository_directory.clone(),
        git: git.into(),
        main_branch: hook.branch.clone(),
    };

    let mut log = SimpleLog::default();

    handle_git_command(
        &["fetch", "origin"],
        "fetching latest changes",
        &mut log,
        &repo,
    )?;
    handle_git_command(
        &["checkout", &repo.main_branch],
        "checking out main branch",
        &mut log,
        &repo,
    )?;
    handle_git_command(
        &["rebase", &format!("origin/{}", &repo.main_branch)],
        "rebasing onto latest changes",
        &mut log,
        &repo,
    )?;
    handle_command(&hook.script, "running hook", &mut log, &repo)?;
    Ok(())
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    let config_file = std::env::args().nth(1).expect("no config argument");
    let config = std::fs::read_to_string(&config_file)
        .with_context(|| format!("Unable to load {}", config_file))?;
    let config: Config = toml::from_str(&config)
        .with_context(|| format!("Unable to parse config file {}", config_file))?;

    for hook in &config.hooks {
        if hook.secret.is_none() {
            eprintln!("WARNING: hook '{}' has no secret specified", hook.name)
        }
    }

    // TODO: Consolidate repos with the same route - they should be OK
    //       we can differentiate them based on what github returns in the
    //       webhook.
    let (send, recv) = std::sync::mpsc::channel::<Event>();

    let mut routes: BTreeMap<String, Route> = BTreeMap::new();

    for hook in config.hooks {
        routes
            .entry(hook.hook_route.clone())
            .or_insert_with(|| Route::new(hook.hook_route.clone(), send.clone()))
            .add_hook(hook);
    }

    let h = std::thread::spawn(
        move || {
            loop {
                match recv.recv().unwrap() {
                    Event::Done => break,
                    Event::PushEvent(event) => {
                        // TODO: We should check the state for this entry in the DB
                        println!("Processing event {}", event.db_id);
                        println!("{:?}", event.content);
                        match update_and_run_hook(&event.hook) {
                            Ok(()) => {}
                            Err(e) => {
                                eprintln!("Error running hook {}: {:?}", event.hook.name, e);
                            }
                        }
                        // TODO: We should update the state for this entry in the DB
                    }
                }
            }
            Ok::<(), std::io::Error>(())
        }
    );

    let mut app = tide::new();
    for (_, route) in routes {
        println!("Adding route = {:?}", route);
        app.at(&route.route()).post(route);
    }

    app.listen("0.0.0.0:8081").await?;

    send.send(Event::Done)?;

    h.join().unwrap().unwrap();

    Ok(())
}
