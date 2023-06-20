#![allow(clippy::upper_case_acronyms, non_snake_case, non_camel_case_types)]

use std::{ffi::c_void, mem::size_of, ptr::null_mut};

pub const INFINITE: u32 = 0xFFFFFFFF;
pub const WAIT_OBJECT_0: u32 = 0x00000000;
pub const STATUS_PENDING: u32 = 0x00000103;

pub type PCWSTR = *const u16;
pub type BOOL = i32;
pub type DWORD = u32;
pub type PDWORD = *mut u32;
pub type PWSTR = *mut u16;
pub type UINT = u32;

type PVOID = *mut c_void;
type HANDLE = *mut c_void;
type WORD = u16;
type PBYTE = *mut u8;

extern "system" {
    pub fn CloseHandle(hObject: HANDLE) -> BOOL;
    pub fn CreateProcessW(
        lpApplicationName: PCWSTR,
        lpCommandLine: PWSTR,
        lpProcessAttributes: *mut SECURITY_ATTRIBUTES,
        lpThreadAttributes: *mut SECURITY_ATTRIBUTES,
        bInheritHandles: BOOL,
        dwCreationFlags: DWORD,
        lpEnvironment: PVOID,
        lpCurrentDirectory: PCWSTR,
        lpStartupInfo: *const STARTUPINFOW,
        lpProcessInformation: *mut PROCESS_INFORMATION,
    ) -> u32;
    pub fn GetExitCodeProcess(hProcess: HANDLE, lpExitCode: PDWORD) -> BOOL;
    pub fn GetLastError() -> DWORD;
    pub fn TerminateProcess(hProcess: HANDLE, uExitCode: UINT) -> BOOL;
    pub fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD;
}

#[repr(C)]
pub struct SECURITY_ATTRIBUTES {
    nLength: DWORD,
    lpSecurityDescriptor: PVOID,
    bInheritHandle: BOOL,
}

#[repr(C)]
pub struct STARTUPINFOW {
    cb: DWORD,
    lpReserved: PWSTR,
    lpDesktop: PWSTR,
    lpTitle: PWSTR,
    dwX: DWORD,
    dwY: DWORD,
    dwXSize: DWORD,
    dwYSize: DWORD,
    dwXCountChars: DWORD,
    dwYCountChars: DWORD,
    dwFillAttribute: DWORD,
    dwFlags: DWORD,
    wShowWindow: WORD,
    cbReserved2: WORD,
    lpReserved2: PBYTE,
    hStdInput: HANDLE,
    hStdOutput: HANDLE,
    hStdError: HANDLE,
}

impl Default for STARTUPINFOW {
    fn default() -> Self {
        Self {
            cb: size_of::<STARTUPINFOW>() as DWORD,
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
            hStdInput: null_mut(),
            hStdOutput: null_mut(),
            hStdError: null_mut(),
        }
    }
}

#[repr(C)]
pub struct PROCESS_INFORMATION {
    pub hProcess: HANDLE,
    pub hThread: HANDLE,
    dwProcessId: DWORD,
    dwThreadId: DWORD,
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
