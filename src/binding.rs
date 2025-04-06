// This type is declared in WinDef.h as `typedef int BOOL;`,
//
// See https://learn.microsoft.com/en-us/windows/win32/winprog/windows-data-types#BOOL
pub(crate) type BOOL = i32;

unsafe extern "system" {
    // Detaches the calling process from its console.
    //
    // See https://learn.microsoft.com/en-us/windows/console/freeconsole
    pub(crate) fn FreeConsole() -> BOOL;
}
