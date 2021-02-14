
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
      "printf" => Function::new_native(&store, api_env_printf),
    },
  };
  
  let instance = Instance::new(&module, &import_object)?;

  //instance.tunables().memory_style();

  // Prep arguments
  let argc = Value::I32( arguments.len() as i32 );


  let argv = Value::I32( 0 );

  // Get the main function and execute it
  let main_fn = instance.exports.get_function("main")?;

  let _result = main_fn.call(&[argc, argv])?;

  Ok(())
}

fn api_env_printf(format_cstr_ptr: i32, va_args_ptr: i32) -> i32 {
  println!("api_env_printf({}, {})", format_cstr_ptr, va_args_ptr);
  // Get a slice that maps to the memory currently used by the webassembly
  // instance.
  //
  // Webassembly only supports a single memory for now,
  // but in the near future, it'll support multiple.
  //
  // Therefore, we don't assume you always just want to access first
  // memory and force you to specify the first memory.
  // let memory = ctx.memory(0);
  // let view: MemoryView<u8> = memory.view();

  // let mut format_str_bytes: Vec<u8> = vec![];

  // for byte in view[format_cstr_ptr .. format_cstr_ptr+4096 /*todo*/].iter().map(Cell::get) {
  //   if byte == 0x00 {
  //     break;
  //   }
  //   format_str_bytes.push(byte);
  // }

  //let format_str = String::from_utf8_lossy(&format_str_bytes);

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

// pub fn get_store(canonicalize_nans: bool) -> Store {
//     Store::new(&get_engine(canonicalize_nans))
// }


use wasmer_engine::Tunables;

use std::ptr::NonNull;

use wasmer::{
    vm::{self, MemoryError, MemoryStyle, TableStyle, VMMemoryDefinition, VMTableDefinition},
    wat2wasm, Memory, MemoryType, Pages, TableType, Target,
    BaseTunables,
};

pub fn get_store_with_middlewares<I: Iterator<Item = Arc<dyn ModuleMiddleware>>>(
    middlewares: I,
) -> Store {
    let mut compiler_config = get_compiler(false);
    for x in middlewares {
        compiler_config.push_middleware(x);
    }
    let engine = Native::new(compiler_config).engine();
    
    let base = BaseTunables::for_target(&Target::default());
    let tunables = LimitingTunables::new(base, Pages(24));

    Store::new_with_tunables(&engine, tunables)
}


/// A custom tunables that allows you to set a memory limit.
///
/// After adjusting the memory limits, it delegates all other logic
/// to the base tunables.
pub struct LimitingTunables<T: Tunables> {
    /// The maxium a linear memory is allowed to be (in Wasm pages, 65 KiB each).
    /// Since Wasmer ensures there is only none or one memory, this is practically
    /// an upper limit for the guest memory.
    limit: Pages,
    /// The base implementation we delegate all the logic to
    base: T,
}

impl<T: Tunables> LimitingTunables<T> {
    pub fn new(base: T, limit: Pages) -> Self {
        Self { limit, base }
    }

    /// Takes in input memory type as requested by the guest and sets
    /// a maximum if missing. The resulting memory type is final if
    /// valid. However, this can produce invalid types, such that
    /// validate_memory must be called before creating the memory.
    fn adjust_memory(&self, requested: &MemoryType) -> MemoryType {
        let mut adjusted = requested.clone();
        if requested.maximum.is_none() {
            adjusted.maximum = Some(self.limit);
        }
        adjusted
    }

    /// Ensures the a given memory type does not exceed the memory limit.
    /// Call this after adjusting the memory.
    fn validate_memory(&self, ty: &MemoryType) -> Result<(), MemoryError> {
        if ty.minimum > self.limit {
            return Err(MemoryError::Generic(
                "Minimum exceeds the allowed memory limit".to_string(),
            ));
        }

        if let Some(max) = ty.maximum {
            if max > self.limit {
                return Err(MemoryError::Generic(
                    "Maximum exceeds the allowed memory limit".to_string(),
                ));
            }
        } else {
            return Err(MemoryError::Generic("Maximum unset".to_string()));
        }

        Ok(())
    }
}

impl<T: Tunables> Tunables for LimitingTunables<T> {
    /// Construct a `MemoryStyle` for the provided `MemoryType`
    ///
    /// Delegated to base.
    fn memory_style(&self, memory: &MemoryType) -> MemoryStyle {
        let adjusted = self.adjust_memory(memory);
        self.base.memory_style(&adjusted)
    }

    /// Construct a `TableStyle` for the provided `TableType`
    ///
    /// Delegated to base.
    fn table_style(&self, table: &TableType) -> TableStyle {
        self.base.table_style(table)
    }

    /// Create a memory owned by the host given a [`MemoryType`] and a [`MemoryStyle`].
    ///
    /// The requested memory type is validated, adjusted to the limited and then passed to base.
    fn create_host_memory(
        &self,
        ty: &MemoryType,
        style: &MemoryStyle,
    ) -> Result<Arc<dyn vm::Memory>, MemoryError> {
        let adjusted = self.adjust_memory(ty);
        self.validate_memory(&adjusted)?;
        self.base.create_host_memory(&adjusted, style)
    }

    /// Create a memory owned by the VM given a [`MemoryType`] and a [`MemoryStyle`].
    ///
    /// Delegated to base.
    unsafe fn create_vm_memory(
        &self,
        ty: &MemoryType,
        style: &MemoryStyle,
        vm_definition_location: NonNull<VMMemoryDefinition>,
    ) -> Result<Arc<dyn vm::Memory>, MemoryError> {
        let adjusted = self.adjust_memory(ty);
        self.validate_memory(&adjusted)?;
        self.base
            .create_vm_memory(&adjusted, style, vm_definition_location)
    }

    /// Create a table owned by the host given a [`TableType`] and a [`TableStyle`].
    ///
    /// Delegated to base.
    fn create_host_table(
        &self,
        ty: &TableType,
        style: &TableStyle,
    ) -> Result<Arc<dyn vm::Table>, String> {
        self.base.create_host_table(ty, style)
    }

    /// Create a table owned by the VM given a [`TableType`] and a [`TableStyle`].
    ///
    /// Delegated to base.
    unsafe fn create_vm_table(
        &self,
        ty: &TableType,
        style: &TableStyle,
        vm_definition_location: NonNull<VMTableDefinition>,
    ) -> Result<Arc<dyn vm::Table>, String> {
        self.base.create_vm_table(ty, style, vm_definition_location)
    }
}



