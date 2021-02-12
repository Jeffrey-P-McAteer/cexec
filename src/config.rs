
use serde::{Serialize, Deserialize};

use std::path::{Path};
use std::fs;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
  pub pid_broadcast_frequency: Duration,
}

impl Default for Config {
  fn default() -> Self {
    Config {
      pid_broadcast_frequency: Duration::from_secs(300),
    }
  }
}

pub fn read_config() -> Config {
  match dirs::config_dir() {
    Some(user_config_d) => {
      let config_path = user_config_d.join("cexec.toml");

      if ! config_path.exists() {
        // Try to write the default config for users to edit later
        if let Err(e) = fs::write(&config_path, include_str!("cexec.toml")) {
          eprintln!("{}", e);
        }
      }

      match read_config_from_file(&config_path) {
        Ok(config) => config,
        Err(e) => {
          eprintln!("{}", e);
          Config::default()
        }
      }
    }
    None => {
      Config::default()
    }
  }
}

pub fn read_config_from_file(file: &Path) -> Result<Config, Box<dyn std::error::Error>> {
  Ok(
    toml::from_str::<Config>(
      &(fs::read_to_string(file)?)
    )?
  )
}

