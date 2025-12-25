use reqwest::blocking::{Client, multipart};
use std::path::Path;
use std::env;
use crate::ui;

pub fn upload_package(zip_path: &str, incident_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    // load .env variables
    dotenvy::dotenv().ok(); 

    let server_url = env::var("SERVER_URL").map_err(|_| "SERVER_URL not set in .env")?;
    let api_key = env::var("API_KEY").map_err(|_| "API_KEY not set in .env")?;

    let path = Path::new(zip_path);
    if !path.exists() {
        return Err("Refined ZIP file not found!".into());
    }

    let form = multipart::Form::new()
        .text("incident_id", incident_id.to_string())
        .file("file", path)?;

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()?;
    
    ui::info(&format!("Connecting to TraceNexus Server at {}...", server_url));

    let response = client.post(&server_url)
        .header("X-TraceNexus-Key", api_key)
        .multipart(form)
        .send();

    match response {
        Ok(res) => {
            if res.status().is_success() {
                ui::success(&format!("Upload successful! Case {} is now being processed by Trace-Nexus.\n", incident_id));
            } else {
                ui::error(&format!("Server reached, but returned error: {}", res.status()));
            }
        },
        Err(e) => {
            if e.is_connect() || e.is_timeout() {
                ui::error(&format!("ERROR: Server is not reachable! (Check your VPN/Internet or if the Pi is online)"));
            } else {
                ui::error(&format!("An unexpected network error occurred: {}", e));
            }
        }
    }

    Ok(())
}