pub mod linux;
pub mod macos;

pub fn execute() -> Result<(), Box<dyn std::error::Error>> {
    if cfg!(target_os = "macos") {
        macos::execute()
    } else if cfg!(target_os = "linux") {
        linux::execute()
    } else {
        Err("Unsupported platform".into())
    }
}
