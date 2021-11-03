#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]
mod config;
mod mod_manager;

use std::path::Path;
use std::sync::Arc;
use sysinfo::{System, SystemExt};
use tokio::sync::Mutex;
use tauri::{State};
use walkdir::WalkDir;
use crate::config::Config;
use crate::mod_manager::Mod;

type GlobalConfig = Arc<Mutex<Config>>;
const AMONG_US_STEAM_ID : &'static str = "945360";

#[tauri::command]
async fn play(window: tauri::Window, config: State<'_, GlobalConfig>) -> Result<(), String> {
    let mut config = config.lock().await;
    // Uninstall old mods
    for index in 0..config.downloaded.len() {
        let mut modification = std::mem::take(&mut config.downloaded[index]);
        window.emit("progress", format!("Removing {}", modification.name)).unwrap();
        match modification.uninstall(&config).await {
            Ok(_) => config.downloaded[index] = modification,
            Err(s) => {config.downloaded[index] = modification; return Err(s)}
        }
    }
    config.save();
    // Download and Install
    for index in 0..config.downloaded.len() {
        let mut modification = std::mem::take(&mut config.downloaded[index]);
        match modification.download(&config, &window).await {
            Ok(_) => {},
            Err(s) => {config.downloaded[index] = modification; return Err(s)}
        }
        window.emit("progress", format!("Installing {}", modification.name)).unwrap();
        modification.install(&config).await?;
        config.downloaded[index] = modification;
    }
    config.save();
    // Start Among Us
    window.emit("progress", format!("Sussing ...")).unwrap();
    if config.run_with_steam {
        tauri::api::shell::open(format!("steam://rungameid/{}", AMONG_US_STEAM_ID), None)
            .or(Err(format!("Could not start among us")))?;
    } else {
        tauri::api::shell::open(Path::new(&config.among_us_path).join("Among Us.exe").display().to_string(), None)
            .or(Err(format!("Could not start among us")))?;
    }
    Ok(())
}

#[tauri::command]
async fn update_mod_config(index : usize, mut new_mod : Mod, config: State<'_, GlobalConfig>) -> Result<(), String> {
    let mut config = config.lock().await;
    if config.downloaded[index].enabled && !new_mod.enabled {
        new_mod.do_uninstall = true;
    }
    config.downloaded[index] = new_mod;
    config.save();
    Ok(())
}

#[tauri::command]
async fn get_mods(config: State<'_, GlobalConfig>) -> Result<Vec<Mod>, String> {
    Ok(config.lock().await.downloaded.clone())
}

#[tauri::command]
async fn add_mod(name : String, location : String, version: String, config: State<'_, GlobalConfig>) -> Result<(), String> {
    let new_mod = Mod::new(name, location, version).await?;
    let mut config = config.lock().await;
    config.downloaded.push(new_mod);
    config.save();
    Ok(())
}

#[tauri::command]
async fn is_among_us_running() -> bool {
    let mut sys = System::new();
    sys.refresh_processes();
    !sys.process_by_name("Among Us").is_empty()
}

fn copy_folder(from : &Path, to : &Path) {
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
          Err(e) => {eprintln!("{}", e)}
        }
    }
}


#[tokio::main]
async fn main() {
    let mut config = Config::load();
    for modification in &mut config.downloaded {
        modification.update_newest_version().await;
    }
    let config = Arc::new(Mutex::new(config));
    tauri::Builder::default()
        .manage(config.clone())
        .on_page_load(move |window, _| {
            let config_clone = config.clone();
            tokio::spawn(async move {
                let mut config = config_clone.lock().await;
                if config.among_us_path.len() == 0 {
                    match config::find_among_us_path(&window) {
                        Some(among_us_path) => {
                            config.among_us_path = among_us_path;
                            config.save();
                        },
                        None => {}
                    }
                }
                // Backup among us folder
                window.emit("load","Backing up").unwrap();
                if !std::path::Path::new(&config.backup_among_us_path).exists() {
                    copy_folder(Path::new(&config.among_us_path), &Path::new(&config.backup_among_us_path));
                }
                window.emit("load","done").unwrap();
            });
        })
        .invoke_handler(tauri::generate_handler!(play, get_mods, update_mod_config, add_mod, is_among_us_running))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
