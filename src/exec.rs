use std::str;
use std::path::Path;
use std::process::{Command, Stdio, Output};
use std::sync::mpsc::Receiver;
use std::sync::Arc;

use conf::{Conf, Project};
use error::{Result, Error};

pub struct Executor {
    conf: Arc<Conf>,
    rx: Receiver<String>,
}

impl Executor {
    pub fn new(conf: Arc<Conf>, rx: Receiver<String>) -> Executor {
        Executor {
            conf: conf,
            rx: rx,
        }
    }

    pub fn start(&self) {
        loop {
            match self.rx.recv() {
                Ok(repo) => self.run(&repo),
                Err(err) => error!("Error occurred while reading updates: {}", err),
            }
        }
    }

    pub fn run(&self, repo: &str) {
        match self.update_repo(repo) {
            Ok(_) => info!("Repository {} updated successfully", repo),
            Err(err) => error!("Failed to update repository {}: {}", repo, err),
        }
    }

    fn update_repo(&self, repo: &str) -> Result<()> {
        let project = try!(self.get_project(repo));
        update_project(&self.conf.location, &self.conf.gitpath, project)
    }

    fn get_project(&self, repo: &str) -> Result<&Project> {
        self.conf
            .get_project(repo)
            .ok_or(Error::from("No repository found"))
    }
}

fn update_project(location: &str, git: &str, project: &Project) -> Result<()> {
    let path_buf = Path::new(location).join(&project.repo);
    let path = path_buf.as_path();
    let _ = try!(git_checkout(git, path, &project.branch));
    let _ = try!(git_pull(git, path));
    run_command_from_str(&project.command, path)
}

fn git_checkout(git: &str, path: &Path, branch: &str) -> Result<()> {
    info!("Checking out branch {} in {}", branch, path_str(path));
    run_command(Command::new(git)
                    .arg("checkout")
                    .arg(branch)
                    .current_dir(path),
                "git checkout")
}

fn path_str(path: &Path) -> &str {
    path.to_str().unwrap_or("[unprintable path]")
}

fn git_pull(git: &str, path: &Path) -> Result<()> {
    info!("Pulling changes in {}", path_str(path));
    run_command(Command::new(git).arg("pull").current_dir(path), "git pull")
}

fn run_command_from_str(command: &str, path: &Path) -> Result<()> {
    info!("Running update command {} in {}", command, path_str(path));
    run_command(Command::new(command).current_dir(path), command)
}

fn run_command(command: &mut Command, name: &str) -> Result<()> {
    command.stdin(Stdio::null())
           .output()
           .map_err(Error::from)
           .and_then(|out| out_to_res(name, &out))
}

fn out_to_res(cmd: &str, out: &Output) -> Result<()> {
    if out.status.success() {
        Ok(())
    } else {
        let text = str::from_utf8(&out.stderr).unwrap_or("[invalid string]");
        let msg = format!("Command {} exited with status {}: {}",
                          cmd,
                          out.status,
                          text);
        Err(Error::from(msg))
    }
}
