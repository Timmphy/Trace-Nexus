use clap::Parser;
use cli::Cli;
use std::io::{self, Write};
use colored::*;
mod cli;
mod admin;
mod profiles;
mod tools;
mod compressor;
mod manifest;
mod refiner;
mod uploader;
mod ui;




fn main() {
    println!("{}", ui::BANNER.bright_cyan().bold());
    println!("{}", " --- Digital Forensics Collector --- \n".bright_cyan().bold());
    ui::info("Initializing TraceNexus engine...");
    let args = Cli::parse();

    // 1. Admin Check
    if !admin::check_admin() {
        ui::error("Run as Administrator!");
        std::process::exit(1);
    }
    // 2. Unblock Tools because Windows is annoying that way
    tools::unblock_tools();

    // 3. Check Tools
    if let Err(missing) = tools::verify_tools() {
        ui::error(&format!("Missing tools: {:?}", missing));
        std::process::exit(1);
    }

    // One output directory for all profiles
    let output_dir = std::env::current_dir().unwrap().join("output");
    std::fs::create_dir_all(&output_dir).ok();
    let output_str = output_dir.to_str().unwrap();

    let profile_name = if args.light { "LIGHT" } else { "FULL" };
    ui::info(&format!("Starting {} collection profile...", profile_name));

    // 1. Data Collection based on profile
    if args.light {
        profiles::run_light(output_str);
    } else if args.full {
        profiles::run_full(output_str);
    }

    ui::info("Generating collection manifest...");
    refiner::run_refinement(output_str);
    
    // ID creation moved to manifest module
    let incident_id = manifest::create_case_summary(output_str);
    
    // ID used for ZIP naming
    compressor::create_packages(output_str, &incident_id);



    ui::warn("Do you want to upload the refined data to the server? (y/N):\n");
    io::stdout().flush().unwrap(); // show text before input

    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();

    if input.trim().to_lowercase() == "y" {
        // Path for refined ZIP
        let zip_name = format!("{}_refined.zip", incident_id);
        let refined_zip_path = output_dir.join(zip_name);
        
        let path_str = refined_zip_path.to_str().unwrap();
        ui::info(&format!("Starting secure transmission to server: {}", path_str));
        if let Err(e) = uploader::upload_package(path_str, &incident_id) {
            ui::info(&format!("Upload process failed: {}", e));
        }
    } else {
        ui::warn("Upload skipped. Data remains local.");
    }
    ui::info(&format!("Collection finished. Original data and ZIPs are stored in: {}", output_str));
    ui::success(&format!("TraceNexus collection finished. Case-ID: {}", incident_id));
}