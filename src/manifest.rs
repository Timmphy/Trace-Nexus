use chrono::Local;
use std::fs;
use std::path::Path;
use serde_json::json;
use std::env;

use crate::ui;

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn create_case_summary(output_dir: &str) -> String {
    let output_path = Path::new(output_dir);
    let refined_path = output_path.join("refined");

    // 1. Get system information from environment variables
    let hostname = env::var("COMPUTERNAME").unwrap_or_else(|_| "Unknown-Host".to_string());
    let username = env::var("USERNAME").unwrap_or_else(|_| "Unknown-User".to_string());
    
    // 2. Get current timestamp
    let now = Local::now();
    let incident_id = format!("INC-{}", now.format("%Y%m%d-%H%M%S"));

    // 3. Create Case Summary JSON
    let summary = json!({
        "case_id": incident_id,
        "generated_at": now.to_rfc3339(),
        "system": {
            "hostname": hostname,
            "user": username,
            "scan_time": now.to_rfc3339(), 
        },
        "collector": {
            "name": "TraceNexus",
            "version": APP_VERSION
        }
    });

    // 4. Save JSON to refined directory
    let summary_path = refined_path.join("case_summary.json");
    let _ = fs::write(summary_path, serde_json::to_string(&summary).unwrap());

    ui::success(&format!("[+] Case Summary created for {} (User: {})", hostname, username));
    incident_id
}