use serde::Serialize;
use std::fs::File;
use std::io::Write;
use chrono::Local;
use std::env;

#[derive(Serialize)]
pub struct Manifest {
    pub computer_name: String,
    pub user_name: String,
    pub collection_time: String,
    pub profile: String,
    pub version: String,
}

pub fn write_manifest(profile: &str, path: &str) {
    // Collect environment information for the manifest like computer name and user name
    let computer = env::var("COMPUTERNAME").unwrap_or_else(|_| "Unknown-PC".to_string());
    let user = env::var("USERNAME").unwrap_or_else(|_| "Unknown-User".to_string());

    let manifest = Manifest {
        computer_name: computer,
        user_name: user,
        collection_time: Local::now().to_rfc3339(),
        profile: profile.to_string(),
        version: "0.1.0".to_string(),
    };

    // create JSON manifest file
    if let Ok(json) = serde_json::to_string_pretty(&manifest) {
        let file_path = format!("{}/manifest.json", path);
        if let Ok(mut file) = File::create(&file_path) {
            let _ = file.write_all(json.as_bytes());
            println!("[+] Manifest created: {}", file_path);
        }
    }
}