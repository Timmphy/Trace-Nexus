use colored::*;

pub const BANNER: &str = r#"
  _______                   _   _                     
 |__   __|                 | \ | |                    
    | |_ __ __ _  ___ ___  |  \| | _____  ___   _ ___ 
    | | '__/ _` |/ __/ _ \ | . ` |/ _ \ \/ / | | / __|
    | | | | (_| | (_|  __/ | |\  |  __/>  <| |_| \__ \
    |_|_|  \__,_|\___\___| |_| \_|\___/_/\_\\__,_|___/v0.2.0
                                                      
                                                      "#;

pub fn info(msg: &str) {
    println!("{} {}", "[*]".bright_blue().bold(), msg.normal());
}

pub fn success(msg: &str) {
    println!("{} {}", "[+]".bright_green().bold(), msg.bold());
}

pub fn error(msg: &str) {
    eprintln!("{} {}", "[-] ERROR:".bright_red().bold(), msg.bright_white());
}

pub fn warn(msg: &str) {
    println!("\n{} {}", "[!]".bright_yellow().bold(), msg.bright_white());
}