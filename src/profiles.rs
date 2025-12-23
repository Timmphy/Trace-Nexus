use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Runs an external forensic tool and logs stdout/stderr.
fn run_command(name: &str, executable: &str, args: Vec<String>, out_dir: &str) {
    println!("[*] Executing: {}", name);

    // 1. Create logs directory
    let log_dir = Path::new(out_dir).join("logs");
    fs::create_dir_all(&log_dir).ok();

    // 2. Get full path of the executable
    let full_exe_path = fs::canonicalize(executable)
        .unwrap_or_else(|_| PathBuf::from(executable));
    
    // Working directory is the executable's directory
    let working_dir = full_exe_path.parent().unwrap_or(Path::new("."));

    // 3. Execute the command
    let output = std::process::Command::new(&full_exe_path)
        .args(&args)
        .current_dir(working_dir)
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);

            // 4. Create log file and write stdout/stderr
            let log_file_path = log_dir.join(format!("{}.log", name));
            if let Ok(mut log_file) = File::create(log_file_path) {
                let _ = writeln!(log_file, "--- STDOUT ---\n{}\n--- STDERR ---\n{}", stdout, stderr);
            }

            // 5. Check exit status
            if out.status.success() {
                println!("[+] {} finished successfully.", name);
            } else {
                println!("[-] {} reported an issue. Check logs/{}.log", name, name);
            }
        },
        Err(e) => eprintln!("[-] Critical Error: Could not start {}: {}", name, e),
    }
}

/// Collects lightweight forensic artifacts for quick triage.
pub fn run_light(out_dir: &str) {
    println!("\n--- [ PHASE: LIGHT COLLECTION ] ---");

    // Amcache: executed programs and installed software (CSV)
    run_command(
        "AmcacheParser",
        "tools/AmcacheParser.exe",
        vec![
            "-f".into(), "C:\\Windows\\AppCompat\\Programs\\Amcache.hve".into(),
            "--csv".into(), out_dir.into(),
            "--mp".into(), 
        ],
        out_dir
    );

    // ShimCache: Binary execution artifacts (CSV)
    run_command(
        "ShimCacheParser",
        "tools/AppCompatCacheParser.exe",
        vec![
            "--csv".into(), out_dir.into(),
        ],
        out_dir
    );

    // Event Logs: System and Security logs (JSON)
    run_command(
        "EvtxECmd",
        "tools/EvtxECmd/EvtxECmd.exe",
        vec![
            "-d".into(), "C:\\Windows\\System32\\winevt\\Logs".into(), 
            "--json".into(), out_dir.into()
        ],
        out_dir
    );
}

/// Collects comprehensive forensic artifacts for deep analysis.
pub fn run_full(out_dir: &str) {
    // Light analysis first
    run_light(out_dir);

    println!("\n--- [ PHASE: FULL DEEP DIVE ] ---");

    // MFT: Master File Table analysis (JSON)
    run_command(
        "MFTECmd",
        "tools/MFTECmd.exe",
        vec![
            "-f".into(), "C:\\$MFT".into(), 
            "--json".into(), out_dir.into()
        ],
        out_dir
    );

    // 2. RECmd with Path to Expert Batch File
    let relative_batch_path = "tools/RECmd/BatchExamples/DFIRBatch.reb";
    
    // We need the absolute path for RECmd
    let absolute_batch_path = std::fs::canonicalize(relative_batch_path)
        .unwrap_or_else(|_| std::path::PathBuf::from(relative_batch_path))
        .to_string_lossy()
        .to_string();

    println!("[*] Loading Expert Batch: {}", relative_batch_path);

    run_command(
        "RECmd_Expert_Batch",
        "tools/RECmd/RECmd.exe",
        vec![
            "-d".into(), "C:\\Windows\\System32\\config".into(),
            "--bn".into(), absolute_batch_path, // now absolute path
            "--csv".into(), out_dir.into()
        ],
        out_dir
    );
}