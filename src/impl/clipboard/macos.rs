#![cfg(target_os = "macos")]
#![allow(unexpected_cfgs)]

use cocoa::base::{id, nil};
use cocoa::foundation::NSString;
use objc::runtime::{Class, Object};
use objc::{msg_send, sel, sel_impl};
use super::Clipboard;
use std::collections::HashMap;

pub struct NSPasteboard {
    pasteboard: *mut Object,
}

impl NSPasteboard {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        unsafe {
            cocoa::appkit::NSApplication::sharedApplication(cocoa::base::nil);
            let cls = Class::get("NSPasteboard").ok_or("Failed to get NSPasteboard class")?;
            let pasteboard: *mut Object = msg_send![cls, generalPasteboard];
            Ok(NSPasteboard { pasteboard })
        }
    }
}

impl Clipboard for NSPasteboard {
    fn get_change_count(&self) -> i32 {
        unsafe { msg_send![self.pasteboard, changeCount] }
    }
    
    fn get_by_type(&mut self, content_type: &str) -> Result<String, Box<dyn std::error::Error>> {
        unsafe {
            let string_type: id = NSString::alloc(nil).init_str(content_type);
            let contents: *mut Object = msg_send![self.pasteboard, stringForType:string_type];
            if contents.is_null() {
                return Err(format!("No content found for type: {}", content_type).into());
            }
            let c_str: *const i8 = msg_send![contents, UTF8String];
            if c_str.is_null() {
                return Err("Failed to get UTF8 string from clipboard content".into());
            }
            Ok(std::ffi::CStr::from_ptr(c_str)
                .to_string_lossy()
                .into_owned())
        }
    }
    
    fn get_string(&mut self) -> Option<String> {
        self.get_by_type("public.utf8-plain-text").ok()
    }
    
    fn set_by_type(&self, _content_type: &str, _content: &str) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Implement clipboard content setting for macOS
        Err("NSPasteboard::set_by_type not yet implemented".into())
    }
    
    fn set_multiple_types(&self, _types: &HashMap<String, String>) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Implement multiple type setting for macOS
        Err("NSPasteboard::set_multiple_types not yet implemented".into())
    }
    
    fn list_types(&self) -> Vec<String> {
        unsafe {
            let types_array: *mut Object = msg_send![self.pasteboard, types];
            if types_array.is_null() {
                return Vec::new();
            }

            let count: usize = msg_send![types_array, count];
            let mut types = Vec::new();

            for i in 0..count {
                let type_obj: *mut Object = msg_send![types_array, objectAtIndex:i];
                if !type_obj.is_null() {
                    let c_str: *const i8 = msg_send![type_obj, UTF8String];
                    if !c_str.is_null() {
                        let type_str = std::ffi::CStr::from_ptr(c_str)
                            .to_string_lossy()
                            .into_owned();
                        types.push(type_str);
                    }
                }
            }

            types
        }
    }

    fn wait(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let initial_change_count = self.get_change_count();
        
        loop {
            std::thread::sleep(std::time::Duration::from_millis(500));
            let current_change_count = self.get_change_count();
            
            if current_change_count != initial_change_count {
                return Ok(());
            }
        }
    }
}
