// https://learn.microsoft.com/en-us/windows/win32/winprog/windows-data-types#BOOL
pub(crate) type BOOL = i32;

extern "system" {
    // https://learn.microsoft.com/en-us/windows/console/freeconsole
    pub(crate) fn FreeConsole() -> BOOL;
}
