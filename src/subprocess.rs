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
    use anyhow::{bail, Result};
    use std::{
        ffi::{c_void, OsStr, OsString},
        iter::once,
        mem::size_of,
        os::windows::ffi::OsStrExt,
        path::{Path, PathBuf},
        ptr::{null, null_mut},
    };

    extern "system" {
        fn GetExitCodeProcess(hProcess: *mut u8, lpExitCode: *mut u32) -> i32;
        fn GetLastError() -> u32;
        fn CloseHandle(hObject: *mut u8) -> u32;
    }

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

            let process_creation_flags = 0;

            let current_directory_ptr = current_directory
                .map(|path| {
                    let wide_path: Vec<u16> =
                        path.as_os_str().encode_wide().chain(once(0)).collect();
                    wide_path.as_ptr()
                })
                .unwrap_or(null_mut());

            let command = command.encode_wide().collect::<Vec<_>>();

            extern "system" {
                fn CreateProcessW(
                    lpApplicationName: *const u16,
                    lpCommandLine: *mut u16,
                    lpProcessAttributes: *mut SECURITY_ATTRIBUTES,
                    lpThreadAttributes: *mut SECURITY_ATTRIBUTES,
                    bInheritHandles: u32,
                    dwCreationFlags: u32,
                    lpEnvironment: *mut u8,
                    lpCurrentDirectory: *const u16,
                    lpStartupInfo: *const STARTUPINFOW,
                    lpProcessInformation: *mut PROCESS_INFORMATION,
                ) -> u32;
            }

            let res = unsafe {
                CreateProcessW(
                    null(),
                    command.as_ptr() as *mut u16,
                    null_mut(),
                    null_mut(),
                    inherit_handles as u32,
                    process_creation_flags,
                    null_mut(),
                    current_directory_ptr,
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
            extern "system" {
                fn TerminateProcess(hProcess: *mut u8, uExitCode: u32) -> i32;
            }

            let res = unsafe { TerminateProcess(self.process_information.hProcess, 0) };

            if res != 0 {
                Ok(())
            } else {
                bail!("Cannot kill process (code {:#x})", unsafe {
                    GetLastError()
                })
            }
        }

        pub fn wait(&self) -> Result<u32> {
            extern "system" {
                fn WaitForSingleObject(hHandle: *mut u8, dwMilliseconds: u32) -> u32;
            }

            let mut exit_code = 0;

            unsafe {
                if WaitForSingleObject(self.process_information.hProcess, 0xFFFFFFFF) == 0x00000000
                {
                    if GetExitCodeProcess(
                        self.process_information.hProcess,
                        &mut exit_code as *mut u32,
                    ) != 0
                    {
                        CloseHandle(self.process_information.hProcess);
                        CloseHandle(self.process_information.hThread);
                        Ok(exit_code)
                    } else {
                        bail!("cannot get exit status (code {:#x})", GetLastError())
                    }
                } else {
                    bail!("cannot wait process (code {:#x})", GetLastError())
                }
            }
        }

        pub fn try_wait(&self) -> Result<Option<u32>> {
            let mut exit_code: u32 = 0;

            unsafe {
                if GetExitCodeProcess(
                    self.process_information.hProcess,
                    &mut exit_code as *mut u32,
                ) != 0
                {
                    if exit_code as i32 == 0x00000103 {
                        Ok(None)
                    } else {
                        CloseHandle(self.process_information.hProcess);
                        CloseHandle(self.process_information.hThread);
                        Ok(Some(exit_code))
                    }
                } else {
                    bail!("cannot get exit status (code {:#x})", GetLastError())
                }
            }
        }
    }

    #[repr(C)]
    struct SECURITY_ATTRIBUTES {
        nLength: u32,
        lpSecurityDescriptor: *mut u8,
        bInheritHandle: u32,
    }

    #[repr(C)]
    struct STARTUPINFOW {
        cb: u32,
        lpReserved: *mut u16,
        lpDesktop: *mut u16,
        lpTitle: *mut u16,
        dwX: u32,
        dwY: u32,
        dwXSize: u32,
        dwYSize: u32,
        dwXCountChars: u32,
        dwYCountChars: u32,
        dwFillAttribute: u32,
        dwFlags: u32,
        wShowWindow: u16,
        cbReserved2: u16,
        lpReserved2: *mut u8,
        hStdInput: u32,
        hStdOutput: u32,
        hStdError: u32,
    }

    impl Default for STARTUPINFOW {
        fn default() -> Self {
            Self {
                cb: size_of::<STARTUPINFOW>() as u32,
                lpReserved: null_mut(),
                lpDesktop: null_mut(),
                lpTitle: null_mut(),
                dwX: 0,
                dwY: 0,
                dwXSize: 0,
                dwYSize: 0,
                dwXCountChars: 0,
                dwYCountChars: 0,
                dwFillAttribute: 0,
                dwFlags: 0,
                wShowWindow: 0,
                cbReserved2: 0,
                lpReserved2: null_mut(),
                hStdInput: 0,
                hStdOutput: 0,
                hStdError: 0,
            }
        }
    }

    #[repr(C)]
    struct PROCESS_INFORMATION {
        hProcess: *mut c_void,
        hThread: *mut c_void,
        dwProcessId: u32,
        dwThreadId: u32,
    }

    impl Default for PROCESS_INFORMATION {
        fn default() -> Self {
            Self {
                hProcess: null_mut(),
                hThread: null_mut(),
                dwProcessId: 0,
                dwThreadId: 0,
            }
        }
    }
}
