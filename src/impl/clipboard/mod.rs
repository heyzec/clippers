use std::collections::HashMap;

/// Trait defining the interface for clipboard operations across different platforms
pub trait Clipboard: std::panic::RefUnwindSafe {
    /// Get the current change count of the clipboard
    fn get_change_count(&self) -> i32;
    
    /// Get clipboard content by MIME type
    fn get_by_type(&mut self, content_type: &str) -> Result<String, Box<dyn std::error::Error>>;
    
    /// Get clipboard content as plain text
    fn get_string(&mut self) -> Option<String>;
    
    /// Set clipboard content by MIME type
    #[allow(dead_code)]
    fn set_by_type(&self, content_type: &str, content: &str) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Set multiple MIME types at once
    #[allow(dead_code)]
    fn set_multiple_types(&self, types: &HashMap<String, String>) -> Result<(), Box<dyn std::error::Error>>;
    
    /// List all available MIME types in the clipboard
    fn list_types(&self) -> Vec<String>;

    /// Wait for the next clipboard change
    fn wait(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Err("wait() not implemented for this platform".into())
    }
}

pub fn create_clipboard() -> Result<Box<dyn Clipboard>, Box<dyn std::error::Error>> {
    #[cfg(target_os = "linux")]
    {
        Ok(Box::new(crate::r#impl::clipboard::linux::LinuxClipboard::new()?))
    }

    #[cfg(target_os = "macos")]
    {
        Ok(Box::new(crate::r#impl::clipboard::macos::NSPasteboard::new()?))
    }
    
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        Err("No clipboard implementation available for this platform".into())
    }
}

pub mod linux;
pub mod macos;
