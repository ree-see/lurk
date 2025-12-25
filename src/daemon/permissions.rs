use anyhow::{anyhow, Result};

#[link(name = "IOKit", kind = "framework")]
unsafe extern "C" {
    fn IOHIDCheckAccess(request_type: u32) -> u32;
    fn IOHIDRequestAccess(request_type: u32) -> bool;
}

const K_IOHID_REQUEST_TYPE_LISTEN_EVENT: u32 = 1;
const K_IOHID_ACCESS_TYPE_GRANTED: u32 = 0;

pub fn check_input_monitoring_permission() -> bool {
    unsafe { IOHIDCheckAccess(K_IOHID_REQUEST_TYPE_LISTEN_EVENT) == K_IOHID_ACCESS_TYPE_GRANTED }
}

pub fn request_input_monitoring_permission() -> bool {
    unsafe { IOHIDRequestAccess(K_IOHID_REQUEST_TYPE_LISTEN_EVENT) }
}

pub fn ensure_permissions() -> Result<()> {
    if !check_input_monitoring_permission() {
        eprintln!("Input Monitoring permission required!");
        eprintln!();
        eprintln!("Please grant permission to 'lurk' in:");
        eprintln!("  System Settings -> Privacy & Security -> Input Monitoring");
        eprintln!();

        request_input_monitoring_permission();

        return Err(anyhow!(
            "Permission denied. Please grant Input Monitoring access and restart."
        ));
    }

    Ok(())
}
