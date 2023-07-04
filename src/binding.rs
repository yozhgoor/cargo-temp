#![allow(clippy::upper_case_acronyms, non_snake_case, non_camel_case_types)]

use std::{ffi::c_void, mem::size_of, ptr::null_mut};

// https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-waitforsingleobject#parameters
pub(crate) const INFINITE: u32 = 0xFFFFFFFF;
// https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-waitforsingleobject#return-value
pub(crate) const WAIT_OBJECT_0: u32 = 0x00000000;
// https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getexitcodeprocess#remarks
pub(crate) const STATUS_PENDING: u32 = 0x00000103;

// https://learn.microsoft.com/en-us/windows/win32/winprog/windows-data-types#BOOL
pub(crate) type BOOL = i32;
// https://learn.microsoft.com/en-us/windows/win32/winprog/windows-data-types#DWORD
pub(crate) type DWORD = u32;
// https://learn.microsoft.com/en-us/windows/win32/winprog/windows-data-types#PCWSTR
pub(crate) type PCWSTR = *const u16;
// https://learn.microsoft.com/en-us/windows/win32/winprog/windows-data-types#PDWORD
pub(crate) type PDWORD = *mut u32;
// https://learn.microsoft.com/en-us/windows/win32/winprog/windows-data-types#PVOID
type PVOID = *mut c_void;
// https://learn.microsoft.com/en-us/windows/win32/winprog/windows-data-types#PWSTR
pub(crate) type PWSTR = *mut u16;
// https://learn.microsoft.com/en-us/windows/win32/winprog/windows-data-types#UINT
pub(crate) type UINT = u32;

// https://learn.microsoft.com/en-us/windows/win32/winprog/windows-data-types#HANDLE
type HANDLE = *mut c_void;
// https://learn.microsoft.com/en-us/windows/win32/winprog/windows-data-types#PBYTE
type PBYTE = *mut u8;
// https://learn.microsoft.com/en-us/windows/win32/winprog/windows-data-types#WORD
type WORD = u16;

extern "system" {
    // https://learn.microsoft.com/en-us/windows/win32/api/handleapi/nf-handleapi-closehandle
    pub(crate) fn CloseHandle(hObject: HANDLE) -> BOOL;
    // https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-createprocessw
    pub(crate) fn CreateProcessW(
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
    ) -> BOOL;
    // https://learn.microsoft.com/en-us/windows/console/freeconsole
    pub(crate) fn FreeConsole() -> BOOL;
    // https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getexitcodeprocess
    pub(crate) fn GetExitCodeProcess(hProcess: HANDLE, lpExitCode: PDWORD) -> BOOL;
    // https://learn.microsoft.com/en-us/windows/win32/api/errhandlingapi/nf-errhandlingapi-getlasterror
    pub(crate) fn GetLastError() -> DWORD;
    // https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-terminateprocess
    pub(crate) fn TerminateProcess(hProcess: HANDLE, uExitCode: UINT) -> BOOL;
    // https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-waitforsingleobject
    pub(crate) fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD;
}

// https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/ns-processthreadsapi-startupinfow
#[repr(C)]
pub(crate) struct STARTUPINFOW {
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

// https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/ns-processthreadsapi-process_information
#[repr(C)]
pub(crate) struct PROCESS_INFORMATION {
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

// https://learn.microsoft.com/en-us/windows/win32/api/wtypesbase/ns-wtypesbase-security_attributes
#[repr(C)]
struct SECURITY_ATTRIBUTES {
    nLength: DWORD,
    lpSecurityDescriptor: PVOID,
    bInheritHandle: BOOL,
}
