use serde::{Serialize, Deserialize};
use crate::Mod;
use std::path::Path;
use std::fs::{File};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub installed : Vec<Mod>,
    pub among_us_path : String,
    pub backup_among_us_path : String,
}

impl Config {

    pub fn load() -> Self {
        let config_path = Path::new("sussy_launcher.json");
        if config_path.exists() {
            let file = File::open(config_path).unwrap();
            serde_json::from_reader(file).unwrap()
        } else {
            let config = Config {
                installed: vec![],
                among_us_path: "".to_string(),
                backup_among_us_path: "".to_string()
            };
            let file = File::create(config_path).unwrap();
            serde_json::to_writer_pretty(file, &config).unwrap();
            config
        }
    }

}