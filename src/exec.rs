
use wasmer::{Store, Module, Instance, Value, Function, Global, imports};
use wasmer_middlewares::Metering;
//use wasmer_runtime::{memory::MemoryView, Ctx, imports, func};

use std::fmt;
use std::cell::Cell;

pub fn run_wasm(
  mut max_cycles: u64,
  wasm_binary: &[u8],
  arguments: Vec<String>
)
  -> Result<(), Box<dyn std::error::Error>>
{

  // We use the middleware example to build a WASM runtime which
  // limits the number of instructions executed.
  // This design makes small, efficient code desirable.
  let store = get_store_with_middlewares(std::iter::once(std::sync::Arc::new(Metering::new(
      max_cycles,
      cost_always_one,
  )) as Arc<dyn wasmer::ModuleMiddleware>));

  let module = Module::new(&store, wasm_binary)?;
  
  // Print all the API arguments the binary is requesting
  // (Good for debugging w/ external languages)
  for import in module.imports() {
    println!("wasm_binary requests import M:{} F:{} T:{:?}", import.module(), import.name(), import.ty());
  }

  // Provide API functions which may call to the host OS
  // for network, filesystem, and GUI access.
  let import_object = imports! {
    "env" => {
      "printf" => func!(api_env_printf),
    },
  };
  
  let instance = Instance::new(&module, &import_object)?;

  // Prep arguments
  let argc = Value::I32( arguments.len() as i32 );


  let argv = Value::I32( 0 );

  // Get the main function and execute it
  let main_fn = instance.exports.get_function("main")?;

  let _result = main_fn.call(&[argc, argv])?;

  Ok(())
}

fn api_env_printf(ctx: &mut Ctx, format_cstr_ptr: i32, va_args_ptr: i32) -> i32 {
  // Get a slice that maps to the memory currently used by the webassembly
  // instance.
  //
  // Webassembly only supports a single memory for now,
  // but in the near future, it'll support multiple.
  //
  // Therefore, we don't assume you always just want to access first
  // memory and force you to specify the first memory.
  let memory = ctx.memory(0);
  let view: MemoryView<u8> = memory.view();

  let mut format_str_bytes: Vec<u8> = vec![];

  for byte in view[format_cstr_ptr .. format_cstr_ptr+4096 /*todo*/].iter().map(Cell::get) {
    if byte == 0x00 {
      break;
    }
    format_str_bytes.push(byte);
  }

  let format_str = String::from_utf8_lossy(&format_str_bytes);

  let mut format_out = String::new();
  //let num_bytes_written = printf_compat::format(&format_str, printf_compat::output::fmt_write(&mut format_out));

  println!("{}", format_out);

  return 0;
}

fn cost_always_one(_: &wasmer::wasmparser::Operator) -> u64 {
    1
}

#[derive(Debug)]
pub struct ExecError {
  pub msg: String
}

impl std::error::Error for ExecError {}

impl ExecError {
  pub fn new<I>(msg: I) -> ExecError where I: Into<String> {
    ExecError {
      msg: msg.into()
    }
  }
  pub fn new_boxed<I>(msg: I) -> Box<ExecError> where I: Into<String> {
    Box::new(ExecError::new(msg))
  }
}

impl fmt::Display for ExecError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ExecError({})", self.msg)
    }
}


use std::sync::Arc;
use wasmer::{ModuleMiddleware};
use wasmer_compiler::CompilerConfig;
use wasmer_engine::Engine;
use wasmer_engine_native::Native;

pub fn get_compiler(canonicalize_nans: bool) -> impl CompilerConfig {
    // Singlepass impl
    let mut compiler = wasmer_compiler_singlepass::Singlepass::new();
    compiler.canonicalize_nans(canonicalize_nans);
    compiler.enable_verifier();
    compiler
}

pub fn get_engine(canonicalize_nans: bool) -> impl Engine {
    let mut compiler_config = get_compiler(canonicalize_nans);
    Native::new(compiler_config).engine()
}

pub fn get_store(canonicalize_nans: bool) -> Store {
    Store::new(&get_engine(canonicalize_nans))
}

pub fn get_store_with_middlewares<I: Iterator<Item = Arc<dyn ModuleMiddleware>>>(
    middlewares: I,
) -> Store {
    let mut compiler_config = get_compiler(false);
    for x in middlewares {
        compiler_config.push_middleware(x);
    }
    let engine = Native::new(compiler_config).engine();
    Store::new(&engine)
}

#[cfg(feature = "test-native")]
pub fn get_headless_store() -> Store {
    Store::new(&Native::headless().engine())
}

