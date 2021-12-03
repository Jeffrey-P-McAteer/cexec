
pub fn run_wasm(
  max_cycles: u64,
  wasm_binary: &[u8],
  arguments: Vec<String>
)
  -> Result<(), Box<dyn std::error::Error>>
{

  // Attempt to turn wast into wasm binary (if fails assume already binary)
  let wasm_binary: Vec<u8> = match wabt::wat2wasm(wasm_binary) {
    Ok(wasm_binary) => wasm_binary,
    Err(e) => wasm_binary.to_vec(),
  };

  let module = wasmi::Module::from_buffer(wasm_binary)?;

  let imports = wasmi::ImportsBuilder::new()
    .with_resolver("env", &EnvModuleResolver);

  let instance = wasmi::ModuleInstance::new(&module, &imports)?;

  let mut host_ex_fns = HostExternalMemory{};

  let instance = instance.not_started_instance(); // TODO should we respect/run start() functions?

  // run "main", assuming it exists
  /*let _res = instance.invoke_export_limited(
    "main",
    &[wasmi::RuntimeValue::I32( arguments.len() as i32 ), wasmi::RuntimeValue::I32( 0 /*TODO copy arguments to WASM memory*/ )],
    &mut host_ex_fns,
    max_cycles
  )?;*/

  let _res = instance.invoke_export(
    "main",
    &[wasmi::RuntimeValue::I32( arguments.len() as i32 ), wasmi::RuntimeValue::I32( 0 /*TODO copy arguments to WASM memory*/ )],
    &mut host_ex_fns
  )?;

  Ok(())
}

// Index table for functions resolved using EnvModuleResolver
const WASM_FN_IDX_printf: usize = 0;

struct EnvModuleResolver;
struct HostExternalMemory {
  // TODO, any API stuff we store for the client WASM blob
}

impl wasmi::ModuleImportResolver for EnvModuleResolver {
  fn resolve_func(&self, field_name: &str, sig: &wasmi::Signature) -> Result<wasmi::FuncRef, wasmi::Error> {
    match field_name {
      "printf" => {
        Ok(wasmi::FuncInstance::alloc_host(
          wasmi::Signature::new(&[wasmi::ValueType::I32, wasmi::ValueType::I32][..], Some(wasmi::ValueType::I32)),
          WASM_FN_IDX_printf
        ))
      }
      unk => {
        Err(wasmi::Error::Instantiation(format!("Unknown function env::{} ( {:?} )", unk, sig)))
      } 
    }
  }
}

impl wasmi::Externals for HostExternalMemory {
    fn invoke_index(
        &mut self,
        index: usize,
        args: wasmi::RuntimeArgs,
    ) -> Result<Option<wasmi::RuntimeValue>, wasmi::Trap> {
        match index {
          WASM_FN_IDX_printf => {
            let a: u32 = args.nth_checked(0)?;
            let b: u32 = args.nth_checked(1)?;
            
            println!("printf({}, {})", a, b); // TODO access to WASM memory

            Ok(Some(wasmi::RuntimeValue::I32( 0 as i32 )))
          }
          unk => {
            eprintln!("Host index {} does not have a function defined!", unk);
            Err(wasmi::Trap::new(wasmi::TrapKind::TableAccessOutOfBounds))
          },
        }
    }
}


