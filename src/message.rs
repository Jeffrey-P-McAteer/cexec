
use crate::*;

/**
 * This enum contains all variants of messages,
 * and the impl is responsible for de/serializing
 * according to addr/002_message_formats.md
 */
#[allow(clippy::large_enum_variant, non_camel_case_types)]
#[derive(Debug)]
pub enum Message {
  NOP { // 0x00
    payload: Vec<u8>,
  },
  PEER_ID_REC { // 0x01
    pub_key: pgp::packet::PublicKey, // owned by the client/server broadcasting their info
    name: String, // 255 byte max
    name_sig: String,
    description: String,
    description_sig: String,
  },
  WASM_EXEC_REQUEST { // 0x02
    pub_key: pgp::packet::PublicKey, // owned by the client submitting the request
    wasm_binary: Vec<u8>,
    wasm_binary_sig: String,
    arguments: Vec<String>,
    exec_req_id: String, // 255 byte max
    exec_req_id_sig: String,
  },
  WASM_EXEC_RESULT { // 0x03
    pub_key: pgp::packet::PublicKey, // owned by the server sending data back
    wasm_stdout: Vec<u8>,
    wasm_stdout_sig: String,
    exec_req_id: String, // 255 byte max
    exec_req_id_sig: String,
  },

}


pub fn get_peer_id_record(c: &config::Config) -> Message {
  use pgp::types::SecretKeyTrait;
  
  Message::PEER_ID_REC {
    pub_key: c.identity_key.primary_key.public_key(),
    name: c.name.clone(),
    name_sig: sign(&c.identity_key, &c.name),
    description: c.description.clone(),
    description_sig: sign(&c.identity_key, &c.description),
  }
}

pub fn sign(identity_key: &pgp::SignedSecretKey, message: &str) -> String {
  String::new()
}
