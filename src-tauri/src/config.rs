use serde::{Serialize, Deserialize};
use std::path::Path;
use std::fs::{File};
use crate::mod_manager::Mod;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub downloaded : Vec<Mod>,
    pub among_us_path : String,
    pub backup_among_us_path : String,
    pub mods_path : String,
    pub run_with_steam : bool
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

}