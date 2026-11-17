use std::{fs, io::ErrorKind, path::PathBuf, time::Duration};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub app_url: String,
    pub app_token: String,
    pub app_name: String,
    pub schedule_timeout: Duration,
    pub api_retry_duration: Duration,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            app_url: "".into(),
            app_token: "".into(),
            app_name: "Roostr".into(),
            schedule_timeout: Duration::from_secs(10),
            api_retry_duration: Duration::from_secs(60),
        }
    }
}

pub struct ConfigManager {
    config: Config,
    path: PathBuf,
}

impl ConfigManager {
    pub fn new() -> ConfigManager {
        let project_dirs = ProjectDirs::from("io", "github", "roostr")
            .expect("Unable to detect config directory.");
        let path = project_dirs.config_dir().join("config.toml");

        let config: Config = match fs::read_to_string(&path) {
            Ok(file) => toml::from_str(&file).expect("Invalid configfile, please check it."),
            Err(error) if error.kind() == ErrorKind::NotFound => {
                let config = Config::default();
                let config_toml = toml::to_string_pretty(&config)
                    .expect("Failed to convert default config to toml.");

                fs::create_dir_all(path.parent().unwrap())
                    .expect("Failed to create configuration folder. Please check permissions.");
                fs::write(&path, config_toml).expect(
                    "Failed to write default config to file system, please check permissions.",
                );

                config
            }
            Err(error) if error.kind() == ErrorKind::PermissionDenied => {
                panic!("Failed to read config file, invalid permissions")
            }
            Err(error) => panic!("Other error related to reading config file. {error}"),
        };

        ConfigManager { config, path }
    }

    pub fn get(&self) -> &Config {
        &self.config
    }

    pub fn update(&mut self, lambda: impl FnOnce(&mut Config)) {
        lambda(&mut self.config);
        self.save();
    }

    fn save(&self) {
        let config_toml =
            toml::to_string_pretty(&self.config).expect("Failed to convert config to toml.");
        fs::write(&self.path, config_toml)
            .expect("Failed to write config file, please check permissions.");
    }
}
