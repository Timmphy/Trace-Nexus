use std::fs::{self, File};
use std::path::{Path};
use walkdir::WalkDir;
use serde_json::{Value, Map};
use chrono::NaiveDateTime;
use chrono::Datelike;
use csv::ReaderBuilder;
use crate::ui;

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
    ui::info("Refining data and cleaning up workspace...");
    
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
            
            // Important: Skip files already in refined or raw directories
            if path.starts_with(&refined_path) || path.starts_with(&raw_path) {
                continue;
            }
            
            // Ignoriere System-JSONs
            if file_name == "manifest.json" || file_name == "case_summary.json" || file_name == "master_timeline.json" {
                continue;
            }

            files_to_process.push(path);
        }
    }

    for path in files_to_process {
        let file_name = path.file_name().unwrap().to_string_lossy().to_string();
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

    // Create timeline after all files are processed
    create_master_timeline(&refined_path);
    cleanup_empty_dirs(base_path);
}

fn cleanup_empty_dirs(path: &Path) {
    // Function to remove empty directories recursively
    for entry in WalkDir::new(path).contents_first(true).into_iter().filter_map(|e| e.ok()) {
        if entry.path().is_dir() {
            let _ = fs::remove_dir(entry.path()); // error if not empty, which is fine
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
                // FIX: to_string() and not to_string_pretty() for minified JSON AI token efficient 
                let _ = fs::write(target_path, serde_json::to_string(&json_data).unwrap());
            }
        },
        "json" => {
            // If it's already JSON, just copy it over
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

pub fn create_master_timeline(refined_path: &Path) {
    let mut timeline = Vec::new();
    let current_year = chrono::Local::now().year(); // Nutzt jetzt Lokalzeit-Jahr

    // Wir gehen durch alle Dateien im refined-Ordner
    for entry in walkdir::WalkDir::new(refined_path).into_iter().filter_map(|e| e.ok()).filter(|e| e.path().is_file()) {
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_string_lossy();
        
        // --- HIER KOMMT DER FILTER HIN ---
        // Wir überspringen alles, was kein JSON ist, UND unsere Spezialdateien
        if !file_name.ends_with(".json") 
           || file_name == "case_summary.json" 
           || file_name == "master_timeline.json" 
        {
            continue;
        }
        // ---------------------------------

        if let Ok(file) = std::fs::File::open(path) {
            let data: serde_json::Value = serde_json::from_reader(file).unwrap_or(serde_json::Value::Null);
            if let Some(array) = data.as_array() {
                for item in array {
                    if let Some(obj) = item.as_object() {
                        if let Some(ts) = find_best_timestamp(obj) {
                            
                            // Nutzt die neue Deep-Scan Logik für die 2069-Treiber
                            let has_future_date = check_for_future_dates(obj, current_year);

                            timeline.push(serde_json::json!({
                                "ts": ts,
                                "cat": path.parent().unwrap().file_name().unwrap().to_string_lossy(),
                                "src": file_name,
                                "suspicious_time": has_future_date, 
                                "data": item
                            }));
                        }
                    }
                }
            }
        }
    }

    // Sortieren und minifiziert speichern
    timeline.sort_by(|a, b| a["ts"].as_str().unwrap().cmp(b["ts"].as_str().unwrap()));
    let _ = std::fs::write(refined_path.join("master_timeline.json"), serde_json::to_string(&timeline).unwrap());
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

// Hilfsfunktion: Scannt ALLE Felder eines Objekts nach verdächtigen Jahreszahlen
fn check_for_future_dates(obj: &serde_json::Map<String, serde_json::Value>, current_year: i32) -> bool {
    for value in obj.values() {
        if let Some(s) = value.as_str() {
            // Wir prüfen, ob der String mit 4 Ziffern beginnt (z.B. "2069-...")
            if s.len() >= 4 && s.chars().take(4).all(|c| c.is_ascii_digit()) {
                if let Ok(year) = s[0..4].parse::<i32>() {
                    // Markieren als verdächtig, wenn das Jahr in der Zukunft liegt
                    // (Aber wir begrenzen es auf 2100, um totalen Datenmüll auszuschließen)
                    if year > current_year && year < 2100 {
                        return true;
                    }
                }
            }
        }
    }
    false
}