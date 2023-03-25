use std::ffi::{CString};
use serde::{Serialize, Deserialize};
use std::path::{Path};
use std::fs::{File};
use sysinfo::{DiskExt, SystemExt};
use tauri::Window;
use walkdir::{WalkDir};
use crate::mod_manager::Mod;
use crate::{KNOWN_MODS, util};

const AMONG_US_PATH_SKIP_DIRS : [&'static str; 26] = ["source", "videos", "images", "docs", "documents", "src", "music", "dev", "windows", "programdata", "lib", "library", "services", "service", "data", "sdk", "packs", "share", "shared", "doc", "required", "bin", "microsoft", "common files", "sysfiles", "content"];
const COMMON_AMONG_US_PATHS : [&'static str; 5] = ["Program Files/Steam/steamapps/common/Among Us/Among Us.exe", "Program Files (x86)/Steam/steamapps/common/Among Us/Among Us.exe", "Program Files/Epic Games/Among Us/Among Us.exe", "Program Files (x86)/Epic Games/Among Us/Among Us.exe", "SteamLibrary/steamapps/common/Among Us/Among Us.exe"];

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub downloaded : Vec<Mod>,
    pub among_us_path : String,
    pub backup_among_us_path : String,
    pub mods_path : String,
    pub run_with_steam : bool
}

fn is_among_us_path(path : &Path) -> bool {
    if !path.exists() {return false}
    if path.extension().is_none() { return false }
    if path.file_name().unwrap().to_str().unwrap() != "Among Us.exe" { return false }
    if path.parent().is_none() { return false }
    let parent_dir_name = path.parent().unwrap().file_name();
    if parent_dir_name.is_none() { return false }
    if parent_dir_name.unwrap() != "Among Us" { return false }
    return true
}

#[cfg(target_family = "windows")]
pub fn find_among_us_path(window : &Window) -> Option<String> {
    let mut sys = sysinfo::System::new();
    sys.refresh_disks_list();
    let disks = sys.disks();
    // Search for common locations first
    for disk in disks {
        let disk_path = disk.mount_point();
        if let Some(among_us_path) = COMMON_AMONG_US_PATHS.iter().find_map(|p| {
            let path = disk_path.join(Path::new(p));
            println!("{}", path.display());
            if is_among_us_path(&path) {
                Some(path)
            } else {
                None
            }
        }) {
            return Some(among_us_path.parent().unwrap().display().to_string());
        }
    }
    // Search basically everywhere
    for disk in disks {
        let disk_path = disk.mount_point();
        println!("Searching disk : {}", disk.mount_point().to_str().unwrap());
        let among_us_path = WalkDir::new(disk_path).into_iter().filter_entry(|e| {
            if e.path().is_dir() {
                !e.file_name().to_str().map(|s|
                    s.starts_with(".") ||
                    s.starts_with("$") ||
                    s.to_lowercase().contains("temp")  ||
                    s.to_lowercase().contains("log")   ||
                    s.to_lowercase().contains("cache") ||
                    s.to_lowercase().contains("assets") ||
                    s.to_lowercase().contains("resources") ||
                    s.to_lowercase().contains("resource") ||
                    s.to_lowercase().contains("node") ||
                    AMONG_US_PATH_SKIP_DIRS.contains(&s.to_lowercase().as_str())
                ).unwrap_or(false)
            } else {true}
        }).filter_map(|e| e.ok()).find(|entry| {
            let path = entry.path();
            if !path.is_file() {
                if entry.depth() <= 2 {
                    window.emit(
                        "load",
                        format!("Searching for Among Us path: {}", path.display())
                    ).unwrap();
                }
                return false
            }
            is_among_us_path(path)
        });
        if among_us_path.is_some() {
            return Some(among_us_path.unwrap().path().parent().unwrap().display().to_string())
        }
    }
    None
}

#[cfg(target_family = "unix")]
fn find_among_us_path(window : &Window) -> Option<String> {
    None
}

#[cfg(target_family = "windows")]
unsafe fn get_dll_version_number<T: AsRef<Path>>(path: T) -> Option<(u16, u16, u16, u16)> {
    let Ok(cstring_path) = CString::new(path.as_ref().display().to_string()) else {return None;};
    let path_pointer = cstring_path.as_ptr() as *const i8;
    let file_version_info_size = winapi::um::winver::GetFileVersionInfoSizeA(path_pointer, std::ptr::null_mut());
    if file_version_info_size == 0 {return None;}
    let mut buffer: Vec<u8> = (0..file_version_info_size).map(|_| 0).collect();
    if winapi::um::winver::GetFileVersionInfoA(
        path_pointer,
        0,
        file_version_info_size,
        buffer.as_mut_ptr() as *mut winapi::ctypes::c_void
    ) == 0 { return None; }
    let minor = (buffer[47] as u16) << 8 | (buffer[48] as u16);
    let major = (buffer[49] as u16) << 8 | (buffer[50] as u16);
    let revision = (buffer[51] as u16) << 8 | (buffer[52] as u16);
    let build = (buffer[53] as u16) << 8 | (buffer[54] as u16);
    Some((major, minor, build, revision))
}

#[cfg(target_family = "unix")]
unsafe fn get_dll_version_number<T: AsRef<Path>>(path: T) -> Option<(u16, u16, u16, u16)> {
    None
}

impl Config {

    pub fn load() -> Self {
        let config_path = Path::new("sussy_launcher.json");
        if config_path.exists() {
            let file = File::open(config_path).unwrap();
            serde_json::from_reader(file).unwrap()
        } else {
            let config = Config {
                downloaded: vec![],
                among_us_path: "".to_string(),
                backup_among_us_path: "./backup".to_string(),
                mods_path : "./mods".to_string(),
                run_with_steam : true
            };
            config.save();
            config
        }
    }

    pub fn save(&self) {
        let config_path = Path::new("sussy_launcher.json");
        let file = File::create(config_path).unwrap();
        serde_json::to_writer_pretty(file, self).unwrap();
    }

     pub async fn add_previously_installed_mods(&mut self, window: &Window) {
        let plugins_path = util::get_plugins_path(self);
        let Ok(read_dir) = plugins_path.read_dir() else {return;};
        let Some(known_mods) = KNOWN_MODS.get() else {return;};
        for entry in read_dir.filter_map(|i| i.ok()) {
            let path = entry.path();
            let Some(file_stem) = path.file_stem() else {continue};
            let Some(file_extension) = path.extension().and_then(|p| p.to_str()) else {continue};
            if file_extension != "dll" {continue;}
            let dll_mod_version = unsafe {get_dll_version_number(&path)};
            let installed_mod_version_string = if let Some(v) = dll_mod_version { format!("{}.{}.{}", v.0, v.1, v.2) } else {"0.0.0".to_string()};
            let mut modification = match known_mods.iter().find(|m| m.name.to_lowercase().as_str() == file_stem.to_ascii_lowercase()) {
                Some(known_mod) => {
                    // Make sure the mod is not already "installed"
                    if self.downloaded.iter()
                        .find(|m| m.name.to_lowercase().as_str() == file_stem.to_ascii_lowercase())
                        .is_some() {
                            continue;
                    }
                    let Ok(mut modification) = Mod::new(
                        known_mod.name.clone(),
                        &known_mod.location,
                        installed_mod_version_string.to_string()
                    ).await else {continue;};
                    modification.version = installed_mod_version_string;
                    modification.update_newest_version().await;
                    modification
                },
                None => {
                    let Ok(mut modification) = Mod::new(
                        file_stem.to_str().unwrap().to_string(),
                        &path.display().to_string(),
                        installed_mod_version_string
                    ).await else {continue;};
                    if modification.download(self, window).await.is_err() {continue;};
                    modification
                }
            };
            modification.enabled = true;
            self.downloaded.push(modification);
        }
    }

}