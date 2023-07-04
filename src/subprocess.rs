use crate::config::Config;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

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
                    Command::new(std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string()));
                process
                    .current_dir(self.working_dir.as_deref().unwrap_or(tmp_dir))
                    .args(["-c", &self.command])
                    .stdin(std::process::Stdio::null());

                if !self.foreground {
                    if !self.stdout.unwrap_or(false) {
                        process.stdout(std::process::Stdio::null());
                    }

                    if !self.stderr.unwrap_or(false) {
                        process.stderr(std::process::Stdio::null());
                    }
                } else {
                    if !self.stdout.unwrap_or(true) {
                        process.stdout(std::process::Stdio::null());
                    }

                    if !self.stderr.unwrap_or(true) {
                        process.stderr(std::process::Stdio::null());
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
                let now = std::time::Instant::now();

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
                    std::thread::sleep(std::time::Duration::from_millis(200));
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

#[cfg(unix)]
pub type Child = std::process::Child;
#[cfg(windows)]
pub use windows::Child;

#[cfg(unix)]
type Command = std::process::Command;
#[cfg(windows)]
pub use windows::Command;

#[cfg(windows)]
mod windows {
    use crate::binding::{
        CloseHandle, CreateProcessW, GetExitCodeProcess, GetLastError, TerminateProcess,
        WaitForSingleObject, BOOL, DWORD, INFINITE, PCWSTR, PDWORD, PROCESS_INFORMATION, PWSTR,
        STARTUPINFOW, STATUS_PENDING, UINT, WAIT_OBJECT_0,
    };
    use anyhow::{bail, Result};
    use std::{
        ffi::{OsStr, OsString},
        iter::once,
        os::windows::ffi::OsStrExt,
        path::{Path, PathBuf},
        ptr::{null, null_mut},
    };

    pub struct Command {
        command: OsString,
        inherit_handles: bool,
        current_directory: Option<PathBuf>,
    }

    impl Command {
        pub fn new(command: impl Into<OsString>) -> Self {
            Self {
                command: command.into(),
                inherit_handles: false,
                current_directory: None,
            }
        }

        pub fn inherit_handles(&mut self, inherit: bool) -> &mut Self {
            self.inherit_handles = inherit;
            self
        }

        pub fn current_dir(&mut self, dir: impl Into<PathBuf>) -> &mut Self {
            self.current_directory = Some(dir.into());
            self
        }

        pub fn spawn(&mut self) -> Result<Child> {
            Child::new(
                self.command.as_ref(),
                self.inherit_handles,
                self.current_directory.as_deref(),
            )
        }

        pub fn status(&mut self) -> Result<u32> {
            self.spawn()?.wait()
        }
    }

    pub struct Child {
        process_information: PROCESS_INFORMATION,
    }

    impl Child {
        fn new(
            command: &OsStr,
            inherit_handles: bool,
            current_directory: Option<&Path>,
        ) -> Result<Self> {
            let startup_info = STARTUPINFOW::default();
            let mut process_info = PROCESS_INFORMATION::default();

            let process_creation_flags = 0 as DWORD;

            let current_directory_ptr = current_directory
                .map(|path| {
                    let wide_path: Vec<u16> =
                        path.as_os_str().encode_wide().chain(once(0)).collect();
                    wide_path.as_ptr()
                })
                .unwrap_or(null_mut());

            let command = command.encode_wide().collect::<Vec<_>>();

            let res = unsafe {
                CreateProcessW(
                    null(),
                    command.as_ptr() as PWSTR,
                    null_mut(),
                    null_mut(),
                    inherit_handles as BOOL,
                    process_creation_flags as DWORD,
                    null_mut(),
                    current_directory_ptr as PCWSTR,
                    &startup_info,
                    &mut process_info,
                )
            };

            if res != 0 {
                Ok(Self {
                    process_information: process_info,
                })
            } else {
                bail!("Cannot create process (code {:#x})", unsafe {
                    GetLastError()
                })
            }
        }

        pub fn kill(&self) -> Result<()> {
            let res = unsafe { TerminateProcess(self.process_information.hProcess, 0 as UINT) };

            if res != 0 {
                Ok(())
            } else {
                bail!("Cannot kill process (code {:#x})", unsafe {
                    GetLastError()
                })
            }
        }

        pub fn wait(&self) -> Result<u32> {
            let mut exit_code = 0;

            let wait = unsafe {
                WaitForSingleObject(self.process_information.hProcess, INFINITE) == WAIT_OBJECT_0
            };

            if wait {
                let res = unsafe {
                    GetExitCodeProcess(self.process_information.hProcess, &mut exit_code as PDWORD)
                };

                if res != 0 {
                    unsafe {
                        CloseHandle(self.process_information.hProcess);
                        CloseHandle(self.process_information.hThread);
                    }

                    Ok(exit_code)
                } else {
                    bail!("cannot get exit status (code {:#x})", unsafe {
                        GetLastError()
                    })
                }
            } else {
                bail!("cannot wait process (code {:#x})", unsafe {
                    GetLastError()
                })
            }
        }

        pub fn try_wait(&self) -> Result<Option<u32>> {
            let mut exit_code: u32 = 0;

            let res = unsafe {
                GetExitCodeProcess(self.process_information.hProcess, &mut exit_code as PDWORD)
            };

            if res != 0 {
                if exit_code == STATUS_PENDING {
                    Ok(None)
                } else {
                    unsafe {
                        CloseHandle(self.process_information.hProcess);
                        CloseHandle(self.process_information.hThread);
                    }

                    Ok(Some(exit_code))
                }
            } else {
                bail!("cannot get exit status (code {:#x})", unsafe {
                    GetLastError()
                })
            }
        }
    }
}
