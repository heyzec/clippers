#![cfg(target_os = "macos")]
#![allow(unexpected_cfgs)] // To suppress warnings when using msg_send!

use super::Clipboard;
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

    fn get_change_count(&self) -> i32 {
        unsafe { msg_send![self.pasteboard, changeCount] }
    }
}

impl Clipboard for NSPasteboard {
    fn get_by_type(&mut self, content_type: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        unsafe {
            let string_type: id = NSString::alloc(nil).init_str(content_type);
            let data: *mut Object = msg_send![self.pasteboard, dataForType:string_type];

            if data.is_null() {
                return Err(format!("No content found for type: {}", content_type).into());
            }

            let length: usize = msg_send![data, length];

            // Handle empty data
            if length == 0 {
                return Ok(Vec::new());
            }

            let bytes: *const u8 = msg_send![data, bytes];

            // Check if bytes pointer is null
            if bytes.is_null() {
                return Err(format!("Invalid data pointer for type: {}", content_type).into());
            }

            let slice = std::slice::from_raw_parts(bytes, length);
            let owned: Vec<u8> = slice.to_vec();

            Ok(owned)
        }
    }

    fn get_string(&mut self) -> Option<String> {
        let bytes = match self.get_by_type("public.utf8-plain-text") {
            Ok(b) => b,
            Err(_) => return None,
        };

        let s = String::from_utf8(bytes).expect("Invalid UTF-8");
        Some(s)
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

    fn set_types(
        &mut self,
        types: &std::collections::HashMap<String, Vec<u8>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            // Clear the pasteboard first
            let _: i32 = msg_send![self.pasteboard, clearContents];

            let ns_data_class = Class::get("NSData").ok_or("Failed to get NSData class")?;

            // Set each type without clearing in between
            for (content_type, content) in types {
                let string_type: id = NSString::alloc(nil).init_str(content_type);

                // Create NSData from bytes
                let ns_data: id = msg_send![ns_data_class,
                    dataWithBytes:content.as_ptr()
                    length:content.len()
                ];

                let success: bool = msg_send![self.pasteboard, setData:ns_data forType:string_type];

                if !success {
                    return Err(format!(
                        "Failed to set clipboard content for type: {}",
                        content_type
                    )
                    .into());
                }
            }

            Ok(())
        }
    }
}
