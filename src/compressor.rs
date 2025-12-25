use std::fs::File;
use std::io::{Write, Read};
use std::path::Path;
use zip::write::SimpleFileOptions; 
use walkdir::WalkDir;
use crate::ui;

pub fn create_packages(output_dir: &str, incident_id: &str) { 
    let base_path = Path::new(output_dir);
    let raw_path = base_path.join("raw");
    let refined_path = base_path.join("refined");

    ui::info(&format!("Starting compression for case: {}...", incident_id));

    // 1. Raw Package e.g. INC-20251224-230313_raw.zip
    if raw_path.exists() {
        let zip_name = format!("{}_raw.zip", incident_id);
        let zip_path = base_path.join(zip_name);
        if let Err(e) = zip_dir(&raw_path, &zip_path) {
            ui::error(&format!("Error zipping raw data: {}", e));
        } else {
            ui::success(&format!("Created: {}", zip_path.file_name().unwrap().to_string_lossy()));
        }
    }

    // 2. Refined Package
    if refined_path.exists() {
        let zip_name = format!("{}_refined.zip", incident_id);
        let zip_path = base_path.join(zip_name);
        if let Err(e) = zip_dir(&refined_path, &zip_path) {
            ui::error(&format!("Error zipping refined data: {}", e));
        } else {
            ui::success(&format!("Created: {}", zip_path.file_name().unwrap().to_string_lossy()));
        }
    }
}

fn zip_dir(src_dir: &Path, dst_file: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(dst_file)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated) //because deflated is very efficient
        .unix_permissions(0o755);

    let mut buffer = Vec::new();

    for entry in WalkDir::new(src_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        // Relative path for the zip archive
        let name = path.strip_prefix(src_dir)?;

        if path.is_file() {
            zip.start_file(name.to_string_lossy(), options)?;
            let mut f = File::open(path)?;
            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
            buffer.clear();
        } else if !name.as_os_str().is_empty() {
            zip.add_directory(name.to_string_lossy(), options)?;
        }
    }

    zip.finish()?;
    Ok(())
}