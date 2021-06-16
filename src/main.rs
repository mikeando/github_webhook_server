use async_trait::async_trait;
use git::CommandOutput;
use git::GitRepositoryError;
use serde::Deserialize;
use tide::Endpoint;
use tide::Request;
use tide::Response;
use tide::StatusCode;
use anyhow::{Context, Result};

use std::ffi::OsStr;

use crate::git::GitRepository;
use crate::github::GithubPushEvent;

pub mod git;
pub mod github;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct HookConfig {
    hook_route: String,
    repository_directory: String,
    script: String,
    branch: String,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct Config {
    hooks: Vec<HookConfig>,
}

pub struct Route {
    config: HookConfig,
}

#[derive(Debug)]
pub enum RouteError {
    TideError(tide::Error),
    GitRepositoryError(GitRepositoryError),
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
                write!(self.content, "{}: {}:+:{}\n", timestamp, prefix, l).unwrap();
            } else {
                write!(self.content, "{}: {}:|:{}\n", timestamp, prefix, l).unwrap();
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
        Ok(v) => render_command_output_to_log(log, stage, &v),
        Err(v) => {
            render_command_error_to_log(log, stage, &v);
            eprintln!("Error {} - log follows", stage);
            render_log_to_stderr(log);
            Err(v)?
        }
    };
    Ok(())
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
        self.config.hook_route.clone()
    }

    pub async fn process_request(&self, req: &mut Request<()>) -> Result<(), RouteError> {
        let v: tide::Result<GithubPushEvent> = req.body_json().await;
        let v: GithubPushEvent = match v {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Error decoding GithubPushEvent:\n{:?}\n", e);
                return Err(RouteError::TideError(e));
            }
        };

        eprintln!("Got GithubPushEvent:\n{:#?}\n", v);
        // TODO: Validate that this event is for the repository we care about
        //       and the branches we care about.

        let git = "git";
        let repo = GitRepository {
            repo_dir: self.config.repository_directory.clone(),
            git: git.into(),
            main_branch: self.config.branch.clone(),
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
        handle_command(&self.config.script, "running hook", &mut log, &repo)?;

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

#[async_std::main]
async fn main() -> tide::Result<()> {

    let config_file = std::env::args().nth(1).expect("no config argument");
    let config = std::fs::read_to_string(&config_file).with_context(|| format!("Unable to load {}", config_file))?;
    let config: Config = toml::from_str(&config).with_context(|| format!("Unable to parse config file {}", config_file))?;

    // TODO: Consolidate repos with the same route - they should be OK
    //       we can differentiate them based on what github returns in the
    //       webhook.
    let mut app = tide::new();
    for hook in config.hooks {
        println!("Adding hook = {:?}", hook);
        let route = Route { config: hook };
        app.at(&route.route()).post(route);
    }

    app.listen("0.0.0.0:8081").await?;
    Ok(())
}
