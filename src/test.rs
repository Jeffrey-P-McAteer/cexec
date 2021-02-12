
use tempfile::NamedTempFile;

use std::fs;

// Pull in everything in lib.rs
use crate::*;

#[test]
fn example_config_parses() {
  let mut file = NamedTempFile::new().expect("Could not get a temp file");
  fs::write(file.path(), include_str!("cexec.toml")).expect("Could not write tmp file");
  let c = config::read_config_from_file(&file.path());
  println!("c={:?}", c);
}


