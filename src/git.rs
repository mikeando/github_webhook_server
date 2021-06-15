use std::ffi::OsStr;

#[derive(Debug)]
pub enum GitRepositoryError {
    CommandError(std::io::Error),
    CommandFailed(CommandOutput),
}

#[derive(Debug)]
pub struct GitRepository {
    pub repo_dir: String,
    pub git: String,
    pub main_branch: String,
}

#[derive(Debug)]
pub struct CommandOutput {
    pub output: std::process::Output,
}

impl CommandOutput {
    pub fn write_streams(&self) {
        use std::io::{self, Write};
        println!("status: {}", self.output.status);
        io::stdout().write_all(&self.output.stdout).unwrap();
        io::stderr().write_all(&self.output.stderr).unwrap();
    }

    pub fn format(&self, stage_name: &str) -> String {
        use std::fmt::Write;
        let mut s: String = format!("\n--------\n{}\n--------\n", stage_name);
        writeln!(s, "++ status = {}", self.output.status).unwrap();
        if self.output.stdout.is_empty() {
            writeln!(s, "++ stdout [empty]").unwrap();
        } else {
            writeln!(s, "++ stdout").unwrap();
            writeln!(s, "{}", String::from_utf8_lossy(&self.output.stdout)).unwrap();
        }
        if self.output.stderr.is_empty() {
            writeln!(s, "++ stderr [empty]").unwrap();
        } else {
            writeln!(s, "++ stderr").unwrap();
            writeln!(s, "{}", String::from_utf8_lossy(&self.output.stderr)).unwrap();
        }
        s
    }
}

impl GitRepository {
    pub fn run_command<I, S>(&self, cmd: &str, args: I) -> Result<CommandOutput, GitRepositoryError>
    where
        I: IntoIterator<Item = S> + std::fmt::Debug,
        S: AsRef<OsStr>,
    {
        use std::process::Command;

        println!("Running (in {}) {} {:?}", self.repo_dir, cmd, args);

        let output = Command::new(cmd)
            .current_dir(&self.repo_dir)
            .args(args)
            .output()
            .map_err(GitRepositoryError::CommandError)?;

        let output = CommandOutput { output };

        if !output.output.status.success() {
            return Err(GitRepositoryError::CommandFailed(output));
        }
        Ok(output)
    }

    pub fn run_git_command<I, S>(&self, args: I) -> Result<CommandOutput, GitRepositoryError>
    where
        I: IntoIterator<Item = S> + std::fmt::Debug,
        S: AsRef<OsStr>,
    {
        self.run_command(&self.git, args)
    }
}
