
use dirs;

pub struct Config {

}

impl Default for Config {
  
}

pub fn read_config() -> Config {
  match dirs::config_dir() {

  }
}

pub fn read_config_from_file(file: Path) -> Result<Config> {

}

