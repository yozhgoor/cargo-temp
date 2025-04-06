use crate::config::Config;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::{env, process::Stdio, time};

#[cfg(windows)]
pub use CreateProcessW::{Child, Command};
#[cfg(unix)]
pub use std::process::{Child, Command};

#[derive(Serialize, Deserialize)]
pub struct SubProcess {
    pub command: String,
    pub foreground: bool,
    #[serde(default)]
    pub keep_on_exit: bool,
    pub working_dir: Option<PathBuf>,
    #[cfg(unix)]
    pub stdout: Option<bool>,
    #[cfg(unix)]
    pub stderr: Option<bool>,
    #[cfg(windows)]
    pub inherit_handles: Option<bool>,
}

impl SubProcess {
    pub fn spawn(&self, tmp_dir: &Path) -> Option<Child> {
        let mut process = {
            #[cfg(unix)]
            {
                let mut process =
                    Command::new(env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string()));
                process
                    .current_dir(self.working_dir.as_deref().unwrap_or(tmp_dir))
                    .args(["-c", &self.command])
                    .stdin(Stdio::null());

                if !self.foreground {
                    if !self.stdout.unwrap_or(false) {
                        process.stdout(Stdio::null());
                    }

                    if !self.stderr.unwrap_or(false) {
                        process.stderr(Stdio::null());
                    }
                } else {
                    if !self.stdout.unwrap_or(true) {
                        process.stdout(Stdio::null());
                    }

                    if !self.stderr.unwrap_or(true) {
                        process.stderr(Stdio::null());
                    }
                }

                process
            }
            #[cfg(windows)]
            {
                let mut process = Command::new(&self.command);
                process.current_dir(self.working_dir.as_deref().unwrap_or(tmp_dir));

                if let Some(b) = self.inherit_handles {
                    process.inherit_handles(b);
                }

                process
            }
        };

        if !self.foreground {
            match process.spawn().ok() {
                Some(child) => Some(child).filter(|_| !self.keep_on_exit),
                None => {
                    log::error!("an error occurred within the subprocess");
                    None
                }
            }
        } else {
            match process.status() {
                Ok(_) => None,
                Err(err) => {
                    log::error!(
                        "an error occurred within the foreground subprocess: {}",
                        err
                    );
                    None
                }
            }
        }
    }
}

pub fn start_subprocesses(config: &Config, tmp_dir: &Path) -> Vec<Child> {
    config
        .subprocesses
        .iter()
        .filter_map(|x| x.spawn(tmp_dir))
        .collect::<Vec<Child>>()
}

pub fn kill_subprocesses(subprocesses: &mut [Child]) -> Result<()> {
    #[cfg(unix)]
    {
        use anyhow::Context;

        for subprocess in subprocesses.iter_mut() {
            {
                let now = time::Instant::now();

                unsafe {
                    libc::kill(
                        subprocess
                            .id()
                            .try_into()
                            .context("cannot get process id")?,
                        libc::SIGTERM,
                    );
                }

                while now.elapsed().as_secs() < 2 {
                    std::thread::sleep(time::Duration::from_millis(200));
                    if let Ok(Some(_)) = subprocess.try_wait() {
                        break;
                    }
                }
            }
        }
    }

    for subprocess in subprocesses.iter_mut() {
        match subprocess.try_wait() {
            Ok(Some(_)) => {}
            _ => {
                let _ = subprocess.kill();
                let _ = subprocess.wait();
            }
        }
    }

    Ok(())
}
