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
    MissingConfig,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub address: Address,
    pub cvs: Option<CvsPharmacy>,
    pub debug: Option<bool>,
    pub name: String,
    pub pushover: Option<Pushover>,
    pub walgreens: Option<Walgreens>,
}

impl Config {
    pub fn read<S: AsRef<str>>(path: Option<S>, debug: bool) -> Result<Config, ConfigError> {
        let path = match path {
            Some(p) => String::from(p.as_ref()),
            None => {
                return Err(ConfigError::MissingConfig)
            }
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
            Ok(mut c) => {
                c.debug = Some(debug);
                Ok(c)
            }
            Err(e) => Err(ConfigError::InvalidToml(format!("Invalid toml: {:?}", e))),
        }
    }
}
