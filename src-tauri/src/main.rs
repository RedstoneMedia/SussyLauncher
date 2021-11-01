#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]
mod config;
use serde::{Serialize, Deserialize};
use std::sync::{Mutex, Arc};
use crate::config::Config;
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mod {
  pub name : String,
  pub version : String,
  pub repo : Option<String>,
  pub enabled : bool,
}

type GlobalConfig = Arc<Mutex<Config>>;

#[tauri::command]
fn play(_config: State<'_, GlobalConfig>) -> Result<(), String> {
  Ok(())
}

#[tauri::command]
fn get_mods(config: State<'_, GlobalConfig>) -> Result<Vec<Mod>, String> {
  Ok(config.lock().unwrap().installed.clone())
}


fn main() {
  let config = Arc::new(Mutex::new(Config::load()));
  tauri::Builder::default()
      .manage(config)
      .invoke_handler(tauri::generate_handler!(play, get_mods))
      .run(tauri::generate_context!())
      .expect("error while running tauri application");
}
