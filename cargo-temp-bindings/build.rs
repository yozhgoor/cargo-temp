
fn main() {
    #[cfg(windows)]
    windows::build!(Windows::Win32::SystemServices::FreeConsole);
}
