use objc2_app_kit::NSWorkspace;
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
        // SAFETY: NSWorkspace is documented as thread-safe for reading by Apple.
        unsafe {
            let workspace = NSWorkspace::sharedWorkspace();
            let Some(app) = workspace.frontmostApplication() else {
                return "Unknown".to_string();
            };
            let Some(bundle_id) = app.bundleIdentifier() else {
                return "Unknown".to_string();
            };
            bundle_id.to_string()
        }
    }
}

impl Default for AppTracker {
    fn default() -> Self {
        Self::new()
    }
}
