use serde::Deserialize;
use std::fs;
use std::path::Path;
use toml;

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize)]
pub struct Address {
    pub city: String,
    pub latitude: f32,
    pub longitude: f32,
    pub state: String,
    pub street2: Option<String>,
    pub street: String,
    pub zipcode: u32,
    pub zipcode_ext: Option<u32>,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize)]
pub struct CvsPharmacy {
    pub email: String,
    pub password: String,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize)]
pub struct Pushover {
    pub app_token: String,
    pub device: Option<String>,
    pub user_token: String,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize)]
pub struct Walgreens {
    pub email: String,
    pub password: String,
}

#[derive(Debug)]
pub enum ConfigError {
    FileSystemError(String),
    InvalidToml(String),
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub name: String,
    pub address: Address,
    pub cvs: Option<CvsPharmacy>,
    pub pushover: Option<Pushover>,
    pub walgreens: Option<Walgreens>,
}

impl Config {
    pub fn read(path: Option<String>) -> Result<Config, ConfigError> {
        let path = match path {
            Some(p) => p,
            None => String::from("vaccinate.toml"),
        };

        println!("Reading configuration ({})...", path);

        let contents: String = match fs::read_to_string(Path::new(&path)) {
            Ok(c) => c,
            Err(e) => {
                return Err(ConfigError::FileSystemError(format!(
                    "Failed to read config file: {}. Error: {:?}",
                    path, e
                )))
            }
        };

        match toml::from_str::<Config>(&contents) {
            Ok(c) => Ok(c),
            Err(e) => Err(ConfigError::InvalidToml(format!("Invalid toml: {:?}", e))),
        }
    }
}
