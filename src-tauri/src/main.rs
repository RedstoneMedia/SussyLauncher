#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]
mod config;
mod mod_manager;
mod util;
mod github_api;

use std::path::Path;
use std::sync::Arc;
use sysinfo::{System, SystemExt};
use tokio::sync::Mutex;
use serde::{Serialize, Deserialize};
use tauri::State;
use once_cell::sync::OnceCell;
use crate::config::Config;
use crate::mod_manager::Mod;

type GlobalConfig = Arc<Mutex<Config>>;
const AMONG_US_STEAM_ID : &'static str = "945360";

static KNOWN_MODS : OnceCell<Vec<KnownMod>> = OnceCell::new();

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
        modification.install(&config, &window).await?;
        config.downloaded[index] = modification;
    }
    config.save();
    // Start Among Us
    window.emit("progress", format!("Sussing ...")).unwrap();
    if config.run_with_steam {
        open::that(format!("steam://rungameid/{}", AMONG_US_STEAM_ID))
            .or(Err(format!("Could not start among us")))?;
    } else {
        open::that(Path::new(&config.among_us_path).join("Among Us.exe").display().to_string())
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
    if config.downloaded.iter().any(|m| m.name == new_mod.name) {
        return Err("Name already exists".to_string())
    }
    config.downloaded.push(new_mod);
    config.save();
    Ok(())
}

#[tauri::command]
async fn remove_mod(index : usize, config: State<'_, GlobalConfig>) -> Result<(), String> {
    let mut config = config.lock().await;
    config.save();
    let mut mod_to_remove = config.downloaded.remove(index);
    mod_to_remove.remove(&config).await?;
    Ok(())
}

#[tauri::command]
async fn is_among_us_running() -> bool {
    let mut sys = System::new();
    sys.refresh_processes();
    sys.processes_by_name("Among Us").count() > 0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnownMod {
    name : String,
    location : String
}

#[tauri::command]
async fn get_possible_mods(name : String) -> Vec<KnownMod> {
    return KNOWN_MODS.get().unwrap().iter().filter(|m| {
        m.name
            .to_lowercase()
            .replace(" ", "")
            .starts_with(&name.to_lowercase().replace(" ", ""))
    }).take(20).map(|m| m.clone()).collect()
}

#[tokio::main]
async fn main() {
    let mut config = Config::load();
    for modification in &mut config.downloaded {
        modification.update_newest_version().await;
    }
    KNOWN_MODS.set(util::load_known_mods().await).unwrap();
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
                    util::copy_folder(Path::new(&config.among_us_path), &Path::new(&config.backup_among_us_path));
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                window.emit("load","done").unwrap();
            });
        })
        .invoke_handler(tauri::generate_handler!(
            play,
            get_mods,
            update_mod_config,
            add_mod,
            remove_mod,
            is_among_us_running,
            get_possible_mods))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
