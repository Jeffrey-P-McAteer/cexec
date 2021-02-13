
use tempfile::NamedTempFile;

use std::fs;

// Pull in everything in lib.rs
use crate::*;

#[test]
fn example_config_parses() {
  let file = NamedTempFile::new().expect("Could not get a temp file");
  fs::write(file.path(), include_str!("cexec.toml")).expect("Could not write tmp file");
  let c = config::read_config_from_file(&file.path());
  println!("c={:?}", c);
}


#[test]
fn test_packet_handling() {
  let c = config::Config::default();
  let m = message::get_peer_id_record(&c);
  
  assert!(matches!(m, message::Message::PEER_ID_REC { .. } ));

  match m {
    message::Message::PEER_ID_REC{pub_key, name, name_sig, description, description_sig} => {
      
      assert!(message::check_sig(&pub_key, &name, &name_sig));
      assert!(message::check_sig(&pub_key, &description, &description_sig));

    }
    _unk => panic!()
  }


}


