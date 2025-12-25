#![allow(deprecated)]

use cocoa::base::{id, nil};
use objc::{class, msg_send, sel, sel_impl};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

pub struct AppTracker {
    current_app: Arc<RwLock<String>>,
}

impl AppTracker {
    pub fn new() -> Self {
        let initial_app = Self::get_frontmost_app_internal();
        let current_app = Arc::new(RwLock::new(initial_app));

        let current_app_clone = Arc::clone(&current_app);
        thread::spawn(move || loop {
            let app = Self::get_frontmost_app_internal();
            if let Ok(mut current) = current_app_clone.write() {
                *current = app;
            }
            thread::sleep(Duration::from_millis(500));
        });

        Self { current_app }
    }

    pub fn get_current_app(&self) -> String {
        self.current_app
            .read()
            .map(|app| app.clone())
            .unwrap_or_else(|_| "Unknown".to_string())
    }

    fn get_frontmost_app_internal() -> String {
        unsafe {
            let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
            if workspace == nil {
                return "Unknown".to_string();
            }

            let frontmost_app: id = msg_send![workspace, frontmostApplication];
            if frontmost_app == nil {
                return "Unknown".to_string();
            }

            let bundle_id: id = msg_send![frontmost_app, bundleIdentifier];
            if bundle_id == nil {
                return "Unknown".to_string();
            }

            let utf8: *const libc::c_char = msg_send![bundle_id, UTF8String];
            if utf8.is_null() {
                return "Unknown".to_string();
            }

            std::ffi::CStr::from_ptr(utf8)
                .to_string_lossy()
                .into_owned()
        }
    }
}

impl Default for AppTracker {
    fn default() -> Self {
        Self::new()
    }
}
