use std::collections::HashMap;
use ureq::ResponseExt;

#[tauri::command]
fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
fn check_latest_release_version() -> String {
    if let Ok(res) = ureq::get("https://github.com/agx-hv/1hpsi/releases/latest").call() {
        return res
            .get_uri()
            .to_string()
            .trim_start_matches("https://github.com/agx-hv/1hpsi/releases/tag/v")
            .to_string();
    }
    "".to_string()
}

#[tauri::command]
fn update() -> Vec<HashMap<String, String>> {
    if let Ok(parsed) = fetch_psi() {
        let pm25_data = parsed.chart_1hr_pm25;
        let (north, south, east, west, central) = (
            pm25_data.north.output(),
            pm25_data.south.output(),
            pm25_data.east.output(),
            pm25_data.west.output(),
            pm25_data.central.output(),
        );
        let mut output = vec![];
        for i in 0..24 {
            let mut record = HashMap::new();
            let (n, s, e, w, c) = (
                north[i].1.clone(),
                south[i].1.clone(),
                east[i].1.clone(),
                west[i].1.clone(),
                central[i].1.clone(),
            );
            let mut mean = 0.0;
            for v in [&n, &s, &e, &w, &c] {
                mean += str::parse::<f32>(v).unwrap_or(0.0) / 5.0;
            }
            record.insert("timestamp".to_string(), north[i].0.clone());
            record.insert("Overall".to_string(), mean.round().to_string());
            record.insert("North".to_string(), n);
            record.insert("South".to_string(), s);
            record.insert("East".to_string(), e);
            record.insert("West".to_string(), w);
            record.insert("Central".to_string(), c);
            output.push(record);
        }
        return output;
    }
    vec![]
}

mod network;
mod psi;
use psi::models::PsiResponse;

use tauri::Manager;
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            #[cfg(debug_assertions)] // only include this code on debug builds
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
                window.close_devtools();
            }
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            update,
            get_app_version,
            check_latest_release_version,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Fetches PSI data from the API, optionally for a specific date
/// returns a structured PsiResponse
fn fetch_psi() -> Result<PsiResponse, String> {
    use network::client::get;
    use psi::parser::parse_psi_response;
    use std::time::{SystemTime, UNIX_EPOCH};

    // Construct API path
    let epoch_ns = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let path = format!(
        "https://www.haze.gov.sg/api/airquality/jsondata/{}",
        epoch_ns
    );

    // Send GET request and read raw response
    let body = get(&path).map_err(|e| e.to_string())?;

    let parsed = parse_psi_response(&body).map_err(|e| e.to_string())?;

    // Return the parsed PSI response
    Ok(parsed)
}
