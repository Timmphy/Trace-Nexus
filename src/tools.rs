// src/tools.rs
use std::fs;
use std::path::Path;
use crate::ui;

pub const REQUIRED_TOOLS: &[&str] = &[
    "tools/AmcacheParser.exe",
    "tools/AppCompatCacheParser.exe",
    "tools/EvtxECmd/EvtxECmd.exe", 
    "tools/RECmd/RECmd.exe",
    "tools/MFTECmd.exe",       
];

pub fn verify_tools() -> Result<(), Vec<String>> {
    let mut missing = Vec::new();

    for tool in REQUIRED_TOOLS {
        if !std::path::Path::new(tool).exists() {
            missing.push(tool.to_string());
        }
    }

    if missing.is_empty() {
        Ok(())
    } else {
        Err(missing)
    }
}

pub fn unblock_tools() {
    ui::info("Optimization: Unblocking forensic tools...");
    
    // Check recursively in the "tools" directory
    if let Ok(entries) = fs::read_dir("tools") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // if it's a directory, recurse
                unblock_recursive(&path);
            } else {
                remove_zone_identifier(&path);
            }
        }
    }
}

fn unblock_recursive(dir: &Path) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                unblock_recursive(&path);
            } else {
                remove_zone_identifier(&path);
            }
        }
    }
}

fn remove_zone_identifier(path: &Path) {
    // Mark of Web Alternate Data Stream path because Windows adds this to files downloaded from the internet
    let ads_path = format!("{}:Zone.Identifier", path.to_str().unwrap_or(""));
    
    // We try to remove the ADS stream.
    // If it doesn't exist, we ignore the error.
    let _ = fs::remove_file(ads_path);
}