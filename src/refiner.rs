use std::fs::{self, File};
use std::path::{Path};
use walkdir::WalkDir;
use serde_json::{Value, Map, json};
use chrono::NaiveDateTime;
use chrono::Datelike;
use csv::ReaderBuilder;

const CATEGORIES: &[(&str, &str)] = &[
    // --- EXECUTION ---
    ("Amcache", "Execution"),
    ("AppCompat", "Execution"),
    ("UserAssist", "Execution"),
    ("BamDam", "Execution"),
    ("RADAR", "Execution"),
    ("RecentApps", "Execution"),
    ("Prefetch", "Execution"),
    // --- PERSISTENCE ---
    ("Run", "Persistence"),
    ("Services", "Persistence"),
    ("TaskCache", "Persistence"),
    ("FirewallRules", "Persistence"),
    ("AppPaths", "Persistence"),
    ("ActiveSetup", "Persistence"),
    // --- NETWORKING ---
    ("Tcpip", "Networking"),
    ("KnownNetworks", "Networking"),
    ("NetworkAdapters", "Networking"),
    ("NetworkSetup2", "Networking"),
    ("Wifi", "Networking"),
    // --- DEVICES ---
    ("USB", "Devices"),
    ("SCSI", "Devices"),
    ("MountedDevices", "Devices"),
    ("DeviceClasses", "Devices"),
    // --- USERS & SOFTWARE ---
    ("UserAccounts", "Users"),
    ("SAMBuiltin", "Users"),
    ("ProfileList", "Users"),
    ("Products", "Software"),
    ("Uninstall", "Software"), 
    // --- SYSTEM & REPORTS ---
    ("RECmd_Batch", "System"), 
    ("TimeZoneInfo", "System"),
    ("MFTECmd", "FileSystem"),
    ("VolumeInfoCache", "FileSystem"),
    ("EvtxECmd", "Logs"),
    ("ETW", "Logs"),
];

// Main function to run the refinement process
pub fn run_refinement(output_dir: &str) {
    println!("[*] Refining data and cleaning up workspace...");
    
    let base_path = Path::new(output_dir);
    let refined_path = base_path.join("refined");
    let raw_path = base_path.join("raw");

    fs::create_dir_all(&refined_path).ok();
    fs::create_dir_all(&raw_path).ok();

    let mut files_to_process = Vec::new();
    for entry in WalkDir::new(output_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path().to_path_buf();
        if path.is_file() {
            let file_name = path.file_name().unwrap_or_default().to_string_lossy();
            
            // Security checks to skip certain paths
            if path.starts_with(&refined_path) || path.starts_with(&raw_path) || path.to_string_lossy().contains("logs") {
                continue;
            }
            // Ingore own manifest.json files in the base directory
            if file_name == "manifest.json" && path.parent() == Some(base_path) {
                continue;
            }
            files_to_process.push(path);
        }
    }

    for path in files_to_process {
        let file_name = path.file_name().unwrap().to_string_lossy().to_string();

        // Sort the file into a category based on its name
        let category = CATEGORIES.iter()
            .find(|(key, _)| file_name.to_lowercase().contains(&key.to_lowercase()))
            .map(|(_, cat)| *cat)
            .unwrap_or("Other");

        let target_dir = refined_path.join(category);
        fs::create_dir_all(&target_dir).ok();

        process_file(&path, &target_dir);

        let destination_raw = raw_path.join(&file_name);
        let _ = fs::rename(&path, destination_raw);
    }

    // Create agent briefing and master timeline
    generate_agent_briefing(&refined_path);
    create_master_timeline(&refined_path);

    // Cleanup empty directories in the base path
    cleanup_empty_dirs(base_path);
}

fn cleanup_empty_dirs(path: &Path) {
    // Function to remove empty directories recursively
    for entry in WalkDir::new(path).contents_first(true).into_iter().filter_map(|e| e.ok()) {
        if entry.path().is_dir() {
            let _ = fs::remove_dir(entry.path()); // Schlägt automatisch fehl, wenn nicht leer
        }
    }
}


fn process_file(source: &Path, target_dir: &Path) {
    let extension = source.extension().and_then(|s| s.to_str()).unwrap_or("");
    let file_stem = source.file_stem().unwrap().to_string_lossy();
    let target_path = target_dir.join(format!("{}.json", file_stem));

    match extension {
        "csv" => {
            if let Ok(json_data) = convert_csv_to_json_normalized(source) {
                let _ = fs::write(target_path, serde_json::to_string_pretty(&json_data).unwrap());
            }
        },
        "json" => {
            let _ = fs::copy(source, target_path);
        },
        _ => {}
    }
}

fn convert_csv_to_json_normalized(path: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut rdr = ReaderBuilder::new().has_headers(true).flexible(true).from_reader(file);
    let headers = rdr.headers()?.clone();
    let mut records = Vec::new();
    let time_keywords = ["Time", "Date", "Created", "Executed", "Modified", "LastWrite"];

    for result in rdr.records() {
        let record = result?;
        let mut map = Map::new();


        for (i, header) in headers.iter().enumerate() {
        // use .trim to clean up whitespace
        let mut value = record.get(i).unwrap_or("").trim().to_string();
    
        if time_keywords.iter().any(|&k| header.contains(k)) && !value.is_empty() {
            value = normalize_time(&value);
        }

        map.insert(header.to_string(), Value::String(value));
    }
        records.push(Value::Object(map));
    }
    Ok(Value::Array(records))
}

fn normalize_time(raw_time: &str) -> String {
    let formats = ["%Y-%m-%d %H:%M:%S%.f", "%Y-%m-%dT%H:%M:%S%.fZ", "%Y-%m-%d %H:%M:%S", "%m/%d/%Y %H:%M:%S"];
    for fmt in formats {
        if let Ok(dt) = NaiveDateTime::parse_from_str(raw_time, fmt) {
            return dt.and_utc().to_rfc3339();
        }
    }
    raw_time.to_string()
}

fn generate_agent_briefing(refined_path: &Path) {
    let briefing = json!({
        "status": "Ready for AI Analysis",
        "structure": "Categorized by forensic tactics",
        "timeline_available": true
    });
    let _ = fs::write(refined_path.join("agent_briefing.json"), serde_json::to_string_pretty(&briefing).unwrap());
}

pub fn create_master_timeline(refined_path: &Path) {
    let mut timeline = Vec::new();
    let current_year = chrono::Utc::now().year(); 

    for entry in WalkDir::new(refined_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file()) 
    {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") || 
           path.file_name().unwrap() == "agent_briefing.json" || 
           path.file_name().unwrap() == "master_timeline.json" 
        {
            continue;
        }

        if let Ok(file) = File::open(path) {
            let data: Value = serde_json::from_reader(file).unwrap_or(Value::Null);
            if let Some(array) = data.as_array() {
                for item in array {
                    if let Some(obj) = item.as_object() {
                        if let Some(ts) = find_best_timestamp(obj) {
                            
                            // Check for suspicious future timestamps
                            // Mark as suspicious if year > current_year + 1
                            let is_future = ts.starts_with(|c: char| c.is_ascii_digit()) && 
                                            ts[0..4].parse::<i32>().unwrap_or(0) > current_year + 1;

                            timeline.push(json!({
                                "ts": ts,
                                "cat": path.parent().unwrap().file_name().unwrap().to_string_lossy(),
                                "src": path.file_name().unwrap().to_string_lossy(),
                                "suspicious_time": is_future, 
                                "data": item
                            }));
                        }
                    }
                }
            }
        }
    }

    timeline.sort_by(|a, b| a["ts"].as_str().unwrap().cmp(b["ts"].as_str().unwrap()));
    let _ = fs::write(refined_path.join("master_timeline.json"), 
                      serde_json::to_string_pretty(&timeline).unwrap());
}

fn find_best_timestamp(obj: &Map<String, Value>) -> Option<String> {
    // Prioritized list of timestamp keys because different tools use different conventions and because some timestamps are more reliable
    let priority_keys = [
        "ts_normalized", "LastWriteTimestamp", "Timestamp", 
        "NameKeyLastWrite", "DriverLastWriteTime", "CreatedOn"
    ];
    
    for key in priority_keys {
        if let Some(Value::String(val)) = obj.get(key) {
            // Check if the value looks like a timestamp
            if !val.is_empty() && val.chars().next().unwrap().is_ascii_digit() && val.contains('T') {
                return Some(val.clone());
            }
        }
    }
    None
}