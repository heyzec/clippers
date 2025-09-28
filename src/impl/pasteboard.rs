#![allow(unexpected_cfgs)] // TODO: Comment this

use cocoa::base::{id, nil};
use cocoa::foundation::NSString;
use objc::runtime::{Class, Object};
use objc::{msg_send, sel, sel_impl};

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

    pub fn get_change_count(&self) -> i32 {
        unsafe { msg_send![self.pasteboard, changeCount] }
    }

    pub fn get_by_type(&self, content_type: &str) -> Result<String, Box<dyn std::error::Error>> {
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

    pub fn get_string(&self) -> Option<String> {
        self.get_by_type("public.utf8-plain-text").ok()
    }

    pub fn set_by_type(&self, content_type: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            // Clear the pasteboard first
            let _: i32 = msg_send![self.pasteboard, clearContents];
            
            // Create NSString objects for the type and content
            let string_type: id = NSString::alloc(nil).init_str(content_type);
            let string_content: id = NSString::alloc(nil).init_str(content);
            
            // Set the string for the specified type
            let success: bool = msg_send![self.pasteboard, setString:string_content forType:string_type];
            
            if success {
                Ok(())
            } else {
                Err(format!("Failed to set clipboard content for type: {}", content_type).into())
            }
        }
    }

    pub fn set_multiple_types(&self, types: &std::collections::HashMap<String, String>) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            // Clear the pasteboard first
            let _: i32 = msg_send![self.pasteboard, clearContents];
            
            // Set each type without clearing in between
            for (content_type, content) in types {
                let string_type: id = NSString::alloc(nil).init_str(content_type);
                let string_content: id = NSString::alloc(nil).init_str(content);
                
                let success: bool = msg_send![self.pasteboard, setString:string_content forType:string_type];
                
                if !success {
                    return Err(format!("Failed to set clipboard content for type: {}", content_type).into());
                }
            }
            
            Ok(())
        }
    }

    pub fn list_types(&self) -> Vec<String> {
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
}
