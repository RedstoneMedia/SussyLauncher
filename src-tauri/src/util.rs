use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use reqwest::Client;
use tauri::Window;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use futures_util::StreamExt;
use crate::config::Config;
use crate::KnownMod;

const KNOWN_MODS_GITHUB_URL : &'static str = "https://raw.githubusercontent.com/RedstoneMedia/SussyLauncher/master/src-tauri/known_mods.json";

pub fn copy_folder(from : &Path, to : &Path) {
    for entry in WalkDir::new(from).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() {continue}
        let relative_path = pathdiff::diff_paths(path, from).unwrap();
        let output_path = Path::new(to).join(relative_path);
        if let Some(parent_dir) = output_path.parent() {
            std::fs::create_dir_all(parent_dir).unwrap();
        }
        if output_path.exists() {continue} // Don't overwrite
        match std::fs::copy(path, output_path) {
          Ok(_) => {},
          Err(e) => {eprintln!("cannot copy {} to {} Error: {}", path.display(), to.display(), e)}
        }
    }
}

pub fn get_reqwest_client() -> Client {
    reqwest::ClientBuilder::new()
        .user_agent("SussyLauncher")
        .build()
        .unwrap()
}

pub async fn load_known_mods() -> Vec<KnownMod> {
    let file_path = Path::new("known_mods.json");
    match tokio::fs::read_to_string(file_path).await {
        Ok(json_string) => {
            serde_json::from_str(&json_string).unwrap_or(vec![])
        }
        Err(_) => {
            let client = get_reqwest_client();
            match client.get(KNOWN_MODS_GITHUB_URL).send().await {
                Ok(r) => {
                    if r.status().is_success() {
                        r.json().await.unwrap_or(vec![])
                    } else {
                        eprintln!("Got Status code {}, when attempting to request known mods", r.status());
                        vec![]
                    }
                },
                Err(e) => {
                    eprintln!("Cannot get known mods: {}", e);
                    vec![]
                }
            }
        }
    }
}

pub async fn download_file(client: &Client, download_url: &str, output_file_path: &PathBuf, window: &Window, display_name : &str) -> Result<(), String> {
        // Request file
        let response = match client.get(download_url).send().await {
            Ok(r) => r,
            Err(e) => return Err(e.to_string())
        };
        if !response.status().is_success() {
            return Err(format!("Request returned status code {}", response.status().as_u16()))
        }
        let total_size = response.content_length().unwrap();
        // Download and write file
        let mut stream = response.bytes_stream();
        let mut downloaded: u64 = 0;
        let mut output_file = File::create(output_file_path).await.unwrap();
        while let Some(item) = stream.next().await {
            let chunk = item.or(Err(format!("Error while downloading file")))?;
            output_file.write(&chunk).await.or(Err(format!("Error while writing to file")))?;
            downloaded = total_size.min(downloaded + (chunk.len() as u64));
            window.emit("progress", format!("{} {:.1}%", display_name, (downloaded as f64 / total_size as f64) * 100.0)).unwrap();
        }
        Ok(())
    }

pub fn get_plugins_path(config : &Config) -> PathBuf {
    Path::new(&config.among_us_path).join(Path::new("BepInEx/plugins"))
}
