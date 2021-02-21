
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
fn test_message_peer_id_record() {
  let c = config::Config::default();
  let m = message::build_peer_id_record(&c);
  
  assert!(matches!(m, message::Message::PEER_ID_REC { .. } ));

  match m {
    message::Message::PEER_ID_REC{pub_key, name, name_sig, description, description_sig} => {
      
      assert!(message::check_sig(&pub_key, &name.as_bytes(), &name_sig));
      assert!(message::check_sig(&pub_key, &description.as_bytes(), &description_sig));

    }
    _unk => panic!()
  }

}



#[test]
fn test_message_wasm_exec_req() {
  let c = config::Config::default();

  let test_based_wat = r#"
    (module
     (import "env" "printf" (func $printf (param i32 i32) (result i32)))
     (table 0 anyfunc)
     (memory $0 1)
     (data (i32.const 16) "Hello, %d args.\n\00")
     (export "memory" (memory $0))
     (export "main" (func $main))
     (func $main (param $0 i32) (param $1 i32) (result i32)
      (local $2 i32)
      (i32.store offset=4
       (i32.const 0)
       (tee_local $2
        (i32.sub
         (i32.load offset=4
          (i32.const 0)
         )
         (i32.const 16)
        )
       )
      )
      (set_local $2
       (get_local $0)
      )
      (drop
       (call $printf
        (i32.const 16)
        (get_local $2)
       )
      )
      (i32.store offset=4
       (i32.const 0)
       (i32.add
        (get_local $2)
        (i32.const 16)
       )
      )
      (i32.add
       (get_local $0)
       (i32.const 1)
      )
     )
    )
  "#;

  let m = message::build_wasm_exec_req(&c, test_based_wat.as_bytes(), vec![], "abcd1234");
  
  assert!(matches!(m, message::Message::WASM_EXEC_REQUEST { .. } ));

  match m {
    message::Message::WASM_EXEC_REQUEST{pub_key, wasm_binary, wasm_binary_sig, arguments, arguments_sig, exec_req_id, exec_req_id_sig} => {
      
      assert!(message::check_sig(&pub_key, &wasm_binary, &wasm_binary_sig));
      assert!(message::check_sig(&pub_key, &exec_req_id.as_bytes(), &exec_req_id_sig));
      assert!(message::check_sig(&pub_key, &arguments.join("").as_bytes(), &arguments_sig));

      { // Let normal run run
        let r = exec::run_wasm(c.untrusted_max_cycles, &wasm_binary, arguments.clone());

        if let Err(ref e) = r {
          eprintln!("{}:{}: {}", file!(), line!(), e);
        }

        assert!(r.is_ok());
      }

      { // Limit to 3 instructions and assert failure
        let r = exec::run_wasm(3, &wasm_binary, arguments.clone());

        if let Err(ref e) = r {
          eprintln!("{}:{}: {}", file!(), line!(), e);
        }

        assert!(r.is_err());
      }


    }
    _unk => panic!()
  }

}

