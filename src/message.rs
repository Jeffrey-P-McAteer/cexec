
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
    arguments_sig: String, // sig computed from arguments.join("").as_bytes()
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


pub fn build_peer_id_record(c: &config::Config) -> Message {
  use pgp::types::SecretKeyTrait;
  
  Message::PEER_ID_REC {
    pub_key: c.identity_key.primary_key.public_key(),
    name: c.name.clone(),
    name_sig: sign(&c.identity_key, &c.name.as_bytes()),
    description: c.description.clone(),
    description_sig: sign(&c.identity_key, &c.description.as_bytes()),
  }
}

pub fn build_wasm_exec_req<I>(
  c: &config::Config, wasm_bytes: &[u8], arguments: Vec<String>, exec_req_id: I
) -> Message
  where I: Into<String>
{
  use pgp::types::SecretKeyTrait;

  let exec_req_id: String = exec_req_id.into();

  let arguments_sig = sign(&c.identity_key, arguments.join("").as_bytes());
  let exec_req_id_sig = sign(&c.identity_key, exec_req_id.as_bytes());
  
  Message::WASM_EXEC_REQUEST {
    pub_key: c.identity_key.primary_key.public_key(),
    wasm_binary: wasm_bytes.to_vec(),
    wasm_binary_sig: sign(&c.identity_key, wasm_bytes),
    arguments: arguments,
    arguments_sig: arguments_sig,
    exec_req_id: exec_req_id,
    exec_req_id_sig: exec_req_id_sig,
  }
}

pub fn sign(identity_key: &pgp::SignedSecretKey, message: &[u8]) -> String {
  use pgp::types::SecretKeyTrait;
  use sha2::Digest;

  let mut s = String::new();

  let mut hasher = sha2::Sha256::new();
  hasher.update( message );

  let msg_digest = hasher.finalize();

  let r = identity_key.create_signature(
    || "".to_string(), // TODO support encrypted keys
    pgp::crypto::hash::HashAlgorithm::SHA2_256,
    &msg_digest
  );

  match r {
    Ok(mpi_vec) => {
      
      // TODO what if we have 2 or more MPIs?
      if let Some(mpi) = mpi_vec.get(0) {
        base64::encode_config_buf(mpi, base64::STANDARD, &mut s);
      }
      else {
        eprintln!("{}:{}: mpi_vec={:?}", file!(), line!(), mpi_vec);
      }

    }
    Err(e) => {
      eprintln!("{}:{}:{}", file!(), line!(), e);
    }
  }

  return s;
}

// Returns true IFF the signature is valid
pub fn check_sig(pkey: &pgp::packet::PublicKey, message: &[u8], message_sig: &str) -> bool {
  use pgp::types::PublicKeyTrait;
  use sha2::Digest;

  let mut hasher = sha2::Sha256::new();
  hasher.update( message );

  let msg_digest = hasher.finalize();

  let message_sig_bytes = base64::decode(message_sig).unwrap_or_default();
  
  let r = pkey.verify_signature(
    pgp::crypto::hash::HashAlgorithm::SHA2_256,
    &msg_digest,
    &[ pgp::types::Mpi::from_slice(&message_sig_bytes) ]
  );

  match r {
    Ok(()) => {
      return true
    },
    Err(e) => {
      eprintln!("{}:{}:{}", file!(), line!(), e);
      return false;
    }
  }
}
