
use clap::Parser;
use cli::Cli;
mod cli;
mod admin;
mod profiles;
mod tools;
mod manifest;
mod refiner;


fn main() {
    let args = Cli::parse();

    // 1. Admin Check
    if !admin::check_admin() {
        eprintln!("[-] ERROR: Run as Administrator!");
        std::process::exit(1);
    }
    // 2. Unblock Tools because Windows is annoying that way
    tools::unblock_tools();

    // 3. Check Tools
    if let Err(missing) = tools::verify_tools() {
        eprintln!("[-] ERROR: Missing tools: {:?}", missing);
        std::process::exit(1);
    }

    // One output directory for all profiles
    let output_dir = std::env::current_dir().unwrap().join("output");
    std::fs::create_dir_all(&output_dir).ok();
    let output_str = output_dir.to_str().unwrap();

    let profile_name = if args.light { "LIGHT" } else { "FULL" };

    if args.light {
        profiles::run_light(output_str);
    } else if args.full {
        profiles::run_full(output_str);
    }

    println!("\n[*] Generating collection manifest...");
    manifest::write_manifest(profile_name, output_str);
    
    refiner::run_refinement(output_str);
    println!("[+] Collection finished. Data is in: {}", output_str);
}