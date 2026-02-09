mod core;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
fn main() {
    linux::run();
}

#[cfg(target_os = "windows")]
fn main() {
    windows::run();
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn main() {
    eprintln!("dustrown GUI currently supports Linux and Windows runtimes.");
}
