use std::path::{Path, PathBuf};
use reqwest::Client;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use futures_util::StreamExt;
use tauri::Window;
use crate::config::Config;
use crate::copy_folder;
use crate::mod_manager::ModLocation::{Github, Local};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModLocation {
    Github(String, String),
    Local(String)
}

impl ModLocation {
    pub fn new(location_string : String) -> Self {
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
            Local(location_string)
        }
    }
}

impl Default for ModLocation {
    fn default() -> Self {
        Self::Local("".to_string())
    }
}

async fn make_github_api_request(client : &reqwest::Client, route : String) -> Result<serde_json::Value, String> {
    let response = match client.get(format!("https://api.github.com/{}", route)).send().await {
        Ok(r) => r,
        Err(e) => return Err(e.to_string())
    };
    if !response.status().is_success() {
        return Err(format!("Request returned status code {}", response.status().as_u16()))
    }
    match response.json::<serde_json::Value>().await {
        Ok(j) => Ok(j),
        Err(e) => Err(e.to_string())
    }
}

fn get_reqwest_client() -> Client {
    reqwest::ClientBuilder::new()
        .user_agent("SussyLauncher")
        .build()
        .unwrap()
}

fn get_assets(newest_release: &Value) -> Vec<Value> {
    let mut assets = newest_release.get("assets").unwrap().as_array().unwrap().clone();
    // Only keep assets that are .dll or .zip files
    assets.retain(|v| match v.get("content_type").unwrap().as_str().unwrap() {
        "application/x-msdownload" => true,
        "application/x-zip-compressed" => true,
        _ => false
    });
    // Sort by download count
    assets.sort_by(|a, b| {
        let a_download_count = a.get("download_count").unwrap().as_u64().unwrap();
        let b_download_count = b.get("download_count").unwrap().as_u64().unwrap();
        b_download_count.cmp(&a_download_count)
    });
    assets
}

async fn get_newest_release(client: &Client, username: &String, repository_name: &String) -> Result<Value, String> {
    // Get releases
    let releases_value = make_github_api_request(&client, format!("repos/{}/{}/releases", username, repository_name)).await?;
    let releases = releases_value.as_array().unwrap();
    let newest_release = match releases.iter().find(|v| v.get("draft").unwrap().as_bool().unwrap() == false) {
        Some(r) => r,
        None => return Err("No releases were found".to_string())
    };
    Ok(newest_release.clone())
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

    pub async fn new(mut name : String, location_string : String, mut version : String) -> Result<Self, String> {
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
                let client = get_reqwest_client();
                let repo_json = make_github_api_request(&client, format!("repos/{}/{}", username, repository_name)).await?;
                name = repo_json.as_object().unwrap().get("name").unwrap().as_str().unwrap().to_string();
                let newest_release = get_newest_release(&client, username, repository_name).await?;
                version = newest_release.get("tag_name").unwrap().as_str().unwrap().to_string();
                let assets = get_assets(&newest_release);
                mod_type = match assets[0].get("content_type").unwrap().as_str().unwrap() {
                    "application/x-msdownload" => ModType::Dll,
                    "application/x-zip-compressed" => ModType::Files,
                    t => {return Err(format!("Invalid mod file type : {}", t))}
                };
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
        let update = self.do_update && self.version != self.newest_version && self.enabled;
        let mod_folder = Path::new(&config.mods_path).join(Path::new(&self.name));
        if !mod_folder.exists() {
            tokio::fs::create_dir_all(&mod_folder).await.unwrap();
        } else if update {
            tokio::fs::remove_dir_all(&mod_folder).await.unwrap();
            tokio::fs::create_dir_all(&mod_folder).await.unwrap();
        } else {
            return Ok(()); // Already download
        }

        let output_path  : PathBuf = match &self.location {
            Github(username, repository_name) => {
                let client = get_reqwest_client();
                let newest_release = get_newest_release(&client, username, repository_name).await?;
                let assets = get_assets(&newest_release);
                let mod_asset = &assets[0];
                let output_file_name = mod_asset.get("name").unwrap().as_str().unwrap();
                // Request asset
                println!("Downloading : {}", output_file_name);
                let download_url = mod_asset.get("browser_download_url").unwrap().as_str().unwrap();
                let response = match client.get(download_url).send().await {
                    Ok(r) => r,
                    Err(e) => return Err(e.to_string())
                };
                if !response.status().is_success() {
                    return Err(format!("Request returned status code {}", response.status().as_u16()))
                }
                let total_size = response.content_length().unwrap();
                // Download and write asset
                let mut stream = response.bytes_stream();
                let mut downloaded: u64 = 0;
                let output_file_path = mod_folder.join(output_file_name);
                let mut output_file = File::create(&output_file_path).await.unwrap();
                while let Some(item) = stream.next().await {
                    let chunk = item.or(Err(format!("Error while downloading file")))?;
                    output_file.write(&chunk).await.or(Err(format!("Error while writing to file")))?;
                    downloaded = total_size.min(downloaded + (chunk.len() as u64));
                    window.emit("progress", format!("{} {:.1}%", self.name, (downloaded as f64 / total_size as f64) * 100.0)).unwrap();
                }
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

    pub async fn install(&self, config : &Config) -> Result<(), String> {
        if !self.enabled {return Ok(())}
        let mod_folder = self.get_mod_folder(config);
        match self.mod_type {
            ModType::Files => copy_folder(&mod_folder, &Path::new(&config.among_us_path)),
            ModType::Dll => {
                let plugins_path = get_plugins_path(config);
                if !plugins_path.exists() {
                    // BepInEx does not exist so all the files required to use it would need to be downloaded
                    unimplemented!()
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
        let plugins_path = get_plugins_path(config);
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

    pub async fn update_newest_version(&mut self) {
        self.newest_version = match &self.location {
            Github(username, repository_name) => {
                let client = get_reqwest_client();
                match get_newest_release(&client, username, repository_name).await {
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

fn get_plugins_path(config : &Config) -> PathBuf {
    Path::new(&config.among_us_path).join(Path::new("BepInEx/plugins"))
}