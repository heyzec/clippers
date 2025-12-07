/// Common interface for clipboard operations across different platforms
pub trait Clipboard: std::panic::RefUnwindSafe {
    fn get_by_type(&mut self, content_type: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>>;

    #[allow(dead_code)]
    fn get_string(&mut self) -> Option<String>;

    fn list_types(&self) -> Vec<String>;

    /// Wait for the next clipboard change
    fn wait(&mut self) -> Result<(), Box<dyn std::error::Error>>;

    fn set_types(
        &mut self,
        types: &std::collections::HashMap<String, Vec<u8>>,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

pub fn create_clipboard() -> Result<Box<dyn Clipboard>, Box<dyn std::error::Error>> {
    #[cfg(target_os = "linux")]
    {
        Ok(Box::new(
            crate::r#impl::clipboard::linux::LinuxClipboard::new()?,
        ))
    }

    #[cfg(target_os = "macos")]
    {
        Ok(Box::new(
            crate::r#impl::clipboard::macos::NSPasteboard::new()?,
        ))
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        Err("No clipboard implementation available for this platform".into())
    }
}

pub mod linux;
pub mod macos;
