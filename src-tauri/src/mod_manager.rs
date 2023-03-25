use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::Window;
use crate::config::Config;
use crate::{github_api, util};
use crate::util::copy_folder;
use crate::mod_manager::ModLocation::{Github, Local};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModLocation {
    Github(String, String),
    Local(String)
}

impl ModLocation {
    pub fn new(location_string : &str) -> Self {
        if location_string.starts_with("https://") {
            let url = reqwest::Url::parse(&location_string).unwrap();
            match url.domain().unwrap() {
                "github.com" => {
                    // Get username and repository name from url
                    let path_split : Vec<&str> = url.path().split("/").collect();
                    let username = path_split[1].to_string();
                    let repository_name = path_split[2].to_string();
                    Github(username, repository_name)
                },
                _ => unimplemented!()
            }
        } else {
            Local(location_string.to_string())
        }
    }
}

impl Default for ModLocation {
    fn default() -> Self {
        Self::Local("".to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModType {
    Files,
    Dll
}

impl Default for ModType {
    fn default() -> Self { Self::Files }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Mod {
    pub name : String,
    pub version : String,
    pub newest_version : String,
    pub location : ModLocation,
    pub mod_type : ModType,
    pub enabled : bool,
    pub do_uninstall : bool,
    pub do_update : bool
}

impl Mod {

    pub async fn new(mut name : String, location_string : &str, mut version : String) -> Result<Self, String> {
        if name.len() < 3 {
            return Err("Name is to short".to_string());
        }
        if location_string.len() == 0 {
            return Err("Empty Location".to_string())
        }

        let location = ModLocation::new(location_string);
        let mod_type : ModType;
        match &location {
            ModLocation::Github(username, repository_name) => {
                let client = util::get_reqwest_client();
                let repo_json = github_api::make_github_api_request(&client, format!("repos/{}/{}", username, repository_name)).await?;
                name = repo_json.as_object().unwrap().get("name").unwrap().as_str().unwrap().to_string();
                let newest_release = github_api::get_newest_release(&client, username, repository_name).await?;
                version = newest_release.get("tag_name").unwrap().as_str().unwrap().to_string();
                let assets = github_api::get_assets(&newest_release);
                mod_type = get_mod_type(&assets[0])?;
                if assets.len() == 0 {return Err("No assets found".to_string())}
            },
            ModLocation::Local(path_string) => {
                let path = std::path::Path::new(path_string);
                if path.is_file() {
                    mod_type = match path.extension().unwrap().to_str().unwrap() {
                        "zip" => ModType::Files,
                        "dll" => ModType::Dll,
                        e => {return Err(format!("Invalid mod file extension : {}", e))}
                    };
                } else {
                    return Err(format!("Path for mod {} is not a file", name))
                }
            }
        }
        Ok(Self {
            name,
            version: version.clone(),
            newest_version: version,
            location,
            mod_type,
            enabled: true,
            do_uninstall: false,
            do_update: false
        })
    }

    pub async fn download(&mut self, config : &Config, window : &Window) -> Result<(), String> {
        if !self.enabled { return Ok(()); }
        let update = self.do_update && self.version != self.newest_version && self.enabled;
        let mod_folder = Path::new(&config.mods_path).join(Path::new(&self.name));
        if !mod_folder.exists() {
            tokio::fs::create_dir_all(&mod_folder).await.unwrap();
        } else if update {
            tokio::fs::remove_dir_all(&mod_folder).await.unwrap();
            tokio::fs::create_dir_all(&mod_folder).await.unwrap();
        } else {
            return Ok(()); // Already downloaded
        }

        let output_path  : PathBuf = match &self.location {
            Github(username, repository_name) => {
                let client = util::get_reqwest_client();
                let newest_release = github_api::get_newest_release(&client, username, repository_name).await?;
                let assets = github_api::get_assets(&newest_release);
                let mod_asset = &assets[0];
                self.mod_type = get_mod_type(&mod_asset)?;
                let output_file_name = mod_asset.get("name").unwrap().as_str().unwrap();
                println!("Downloading : {}", output_file_name);
                let download_url = mod_asset.get("browser_download_url").unwrap().as_str().unwrap();
                let output_file_path = mod_folder.join(output_file_name);
                util::download_file(&client, download_url, &output_file_path, window, &self.name).await?;
                output_file_path
            },
            Local(path_string) => {
                let path = Path::new(path_string);
                let name = path.file_name().unwrap().to_str().unwrap();
                let output_path = mod_folder.join(Path::new(name));
                tokio::fs::copy(path, &output_path)
                    .await
                    .or(Err(format!("Could not copy mod to mods folder")))?;
                output_path
            }
        };
        let extension = output_path.extension().unwrap().to_str().unwrap();
        match extension {
            "zip" => {
                println!("Extracting : {}", output_path.display());
                window.emit("progress", format!("Extracting")).unwrap();
                let zip_file = std::fs::File::open(&output_path).or(Err(format!("Cannot open Zip file")))?;
                let mut zip_reader = zip::read::ZipArchive::new(zip_file).or(Err(format!("Cannot read zip file")))?;
                zip_reader.extract(mod_folder).or(Err(format!("Cannot extract zip file")))?;
                tokio::fs::remove_file(output_path).await.or(Err(format!("Could not remove zip file")))?;
            },
            _ => {}
        }
        self.do_update = false;
        self.version = self.newest_version.clone();
        Ok(())
    }

    fn get_mod_folder(&self, config : &Config) -> PathBuf {
        Path::new(&config.mods_path).join(Path::new(&self.name))
    }

    pub async fn install(&self, config : &Config, window : &Window) -> Result<(), String> {
        if !self.enabled {return Ok(())}
        let mod_folder = self.get_mod_folder(config);
        match self.mod_type {
            ModType::Files => copy_folder(&mod_folder, &Path::new(&config.among_us_path)),
            ModType::Dll => {
                let plugins_path = util::get_plugins_path(config);
                if !plugins_path.exists() {
                    // BepInEx is not installed so all the files required to use it will need to be downloaded and extracted
                    // Download
                    let client = util::get_reqwest_client();
                    // TODO: Check if it might also be possible to get BepInEx directly from https://builds.bepis.io/projects/bepinex_be
                    let release = github_api::get_newest_release(&client, &"NuclearPowered".to_string(), &"BepInEx".to_string()).await?;
                    let assets= github_api::get_assets(&release).remove(0);
                    let download_url = assets.get("browser_download_url").unwrap().as_str().unwrap();
                    let among_us_path = Path::new(&config.among_us_path);
                    let zip_file_path = among_us_path.join("BepInEx.zip");
                    util::download_file(&client, download_url, &zip_file_path, window, "BepInEx").await?;
                    // Extract zip
                    println!("Extracting : {}", zip_file_path.display());
                    window.emit("progress", format!("Extracting")).unwrap();
                    let zip_file = std::fs::File::open(&zip_file_path).or(Err(format!("Cannot open Zip file")))?;
                    let mut zip_reader = zip::read::ZipArchive::new(zip_file).or(Err(format!("Cannot read zip file")))?;
                    zip_reader.extract(among_us_path).or(Err(format!("Cannot extract zip file")))?;
                    tokio::fs::remove_file(zip_file_path).await.or(Err(format!("Could not remove zip file")))?;
                    tokio::fs::create_dir_all(&plugins_path).await.or(Err(format!("Cannot create plugins folder")))?;

                    window.emit("progress", format!("Installing {}", self.name)).unwrap();
                }
                // Copy first file from mod folder to plugins path
                let dll_path = mod_folder.read_dir().unwrap().next().unwrap().unwrap().path();
                tokio::fs::copy(
                    &dll_path,
                    plugins_path.join(Path::new(dll_path.file_name().unwrap()))
                ).await.unwrap();
            }
        }
        Ok(())
    }

    pub async fn uninstall(&mut self, config : &Config) -> Result<(), String> {
        if !self.do_uninstall || self.enabled { return Ok(()) }
        let mod_folder = self.get_mod_folder(config);
        if !mod_folder.exists() { return Ok(()) }
        let plugins_path = util::get_plugins_path(config);
        let remove_paths : Vec<PathBuf> = match self.mod_type {
            ModType::Files => {
                let plugins_folder = mod_folder.join(Path::new("BepInEx/plugins"));
                plugins_folder
                    .read_dir()
                    .or(Err(format!("{} has not plugins folder", self.name)))?
                    .map(|file| {
                        let path = file.unwrap().path();
                        let name = path.file_name().unwrap();
                        plugins_path.join(name)
                    }).collect()
            },
            ModType::Dll => {
                let dll_path = mod_folder.read_dir().unwrap().next().unwrap().unwrap().path();
                let name = dll_path.file_name().unwrap();
                vec![plugins_path.join(name)]
            }
        };
        for remove_path in remove_paths {
            if remove_path.exists() {
                tokio::fs::remove_file(remove_path)
                    .await
                    .or(Err(format!("Cannot uninstall {}", self.name)))?
            }
        }
        self.do_uninstall = false;
        Ok(())
    }

    pub async fn remove(&mut self, config : &Config) -> Result<(), String> {
        self.do_uninstall = true;
        self.enabled = false;
        self.uninstall(config).await?;
        let mod_folder = self.get_mod_folder(config);
        if mod_folder.exists() {
            tokio::fs::remove_dir_all(mod_folder).await.or(Err(format!("Could not remove mod folder")))?;
        }
        Ok(())
    }

    pub async fn update_newest_version(&mut self) {
        self.newest_version = match &self.location {
            Github(username, repository_name) => {
                let client = util::get_reqwest_client();
                match github_api::get_newest_release(&client, username, repository_name).await {
                    Ok(newest_release) => {
                        newest_release.get("tag_name").unwrap().as_str().unwrap().to_string()
                    },
                    Err(_) => self.version.clone()
                }
            }
            Local(_) => self.version.clone()
        }
    }
}

fn get_mod_type(asset: &Value) -> Result<ModType, String> {
    Ok(match (asset.get("content_type").unwrap().as_str().unwrap(), asset.get("name").unwrap().as_str().unwrap().split(".").last().unwrap()) {
        ("application/x-msdownload", _) => ModType::Dll,
        ("application/octet-stream", "dll") => ModType::Dll,
        ("application/x-zip-compressed", _) => ModType::Files,
        ("application/zip", _) => ModType::Files,
        (t, _) => { return Err(format!("Invalid mod file type : {}", t)) }
    })
}