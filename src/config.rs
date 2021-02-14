
use serde::{Deserialize, Deserializer};
use pgp::Deserializable;

use std::path::{Path};
use std::fs;
use std::time::Duration;
use std::ffi::OsString;
use std::process::Command;

#[derive(Deserialize, Debug)]
pub struct Config {
  // Server and client node identification data
  pub name: String,
  pub description: String,

  // If a PGP key is not found/known a warning is printed and
  // a temporary identity is generated when config is parsed.
  // The temporary identity will be re-used until the user's
  // temporary directory is emptied (usually by reboots)
  #[serde(deserialize_with = "deserialize_identity_key")] 
  pub identity_key: pgp::SignedSecretKey,

  pub pid_broadcast_frequency: Duration,

  pub untrusted_max_cycles: u64,
  pub trusted_max_cycles: u64,
}

impl Default for Config {
  fn default() -> Self {
    // Get the hostname, defaulting to "Unnamed" if OS or utf-8 errors occur.
    let name = hostname::get()
      .unwrap_or_else(|_| OsString::from("Unnamed"))
      .into_string()
      .unwrap_or_else(|_| "Unnamed".to_string());

    Config {
      name: name,
      description: String::new(),
      identity_key: read_identity_key(None),
      pid_broadcast_frequency: Duration::from_secs(300),
      untrusted_max_cycles: 1222111,
      trusted_max_cycles: 999000111,
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
          eprintln!("{}:{}: {}", file!(), line!(), e);
        }
      }

      match read_config_from_file(&config_path) {
        Ok(config) => config,
        Err(e) => {
          eprintln!("{}:{}: {}", file!(), line!(), e);
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

fn deserialize_identity_key<'de, D>(deserializer: D)
  -> Result<pgp::SignedSecretKey, D::Error>
  where D: Deserializer<'de>
{
  if let Ok(identity_key_s) = Deserialize::deserialize(deserializer) {
    let identity_key_s: &str = identity_key_s;
    Ok(read_identity_key(Some(identity_key_s)))
  }
  else {
    Ok(read_identity_key(None))
  }
}


fn read_identity_key(maybe_armored_string: Option<&str>) -> pgp::SignedSecretKey {
  // Users can copy/paste PGP keys into their config files
  if let Some(maybe_armored_string) = maybe_armored_string {
    match pgp::SignedSecretKey::from_string(maybe_armored_string) {
      Ok((key, _headers)) => return key,
      Err(e) => {
        eprintln!("{}:{}: {}", file!(), line!(), e);
      }
    }
  }

  // We can execute GPG to ask for the first private key
  let gpg_o = Command::new("gpg")
                .args(&["--export-secret-keys", "--armor"])
                .output();
  match gpg_o {
    Ok(gpg_o) => {
      match std::str::from_utf8(&gpg_o.stdout) {
        Ok(maybe_armored_string) => {
          match pgp::SignedSecretKey::from_string(maybe_armored_string) {
            Ok((key, _headers)) => return key,
            Err(e) => {
              eprintln!("{}:{}: {}", file!(), line!(), e);
            }
          }
        }
        Err(e) => {
          eprintln!("{}:{}: {}", file!(), line!(), e);
        }
      }
    }
    Err(e) => {
      eprintln!("{}:{}: {}", file!(), line!(), e);
    }
  }

  // TODO think of more places we can ask the OS for an identity

  todo!()
}


