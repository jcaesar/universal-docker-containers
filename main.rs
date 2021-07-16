use wasmer::{Instance, Module, Store};
use wasmer_compiler_cranelift::Cranelift;
use wasmer_engine_universal::Universal;
use wasmer_wasi::{WasiState, WasiError};
use wasmer_emscripten::{
	generate_emscripten_env, is_emscripten_module, run_emscripten_instance, EmEnv,
	EmscriptenGlobals,
};
use std::env;
use std::process::exit;
use std::path::PathBuf;
use std::os::unix::ffi::{OsStrExt, OsStringExt};


#[cfg(not(unix))]
error!("This is a binfmt interpreter for linux - compiling it for non-unix target makes 0 sense.");

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
	let argv = env::args_os().collect::<Vec<_>>();
	let (_interpreter, executable, original_executable, argv) = match &argv[..] {
		[a, b, c, d@..] => (a,b,c,d),
		_ => {
			eprintln!("Wasmer WASM interpreter needs at least three arguments (including $0) - must be registered as binfmt interpreter with the CFP flags.");
			eprintln!("Got arguments: {:?}", argv);
			exit(-1);
		}
	};
	let wasm_bytes = std::fs::read(executable)?;

	let store = Store::new(&Universal::new(Cranelift::default()).engine());

	let module = Module::new(&store, wasm_bytes)?;

	if is_emscripten_module(&module) {
		let argv = argv.iter()
			.map(|s| -> Result<_> { s.to_str()
				.ok_or_else(|| format!("Can't deal with argument {:?}", s).into())
			}).collect::<Result<Vec<_>>>()?;
		let mut emscripten_globals = EmscriptenGlobals::new(module.store(), &module)?;
		let mut em_env = EmEnv::new(&emscripten_globals.data, Default::default());
		let import_object = generate_emscripten_env(module.store(), &mut emscripten_globals, &mut em_env);
		let mut instance = Instance::new(&module, &import_object)?;
		run_emscripten_instance(
			&mut instance,
			&mut em_env,
			&mut emscripten_globals,
			&original_executable.to_string_lossy(),
			argv.iter().map(|s| s.as_ref()).collect::<Vec<_>>(),
			None,
		)?;
	} else {
		let preopen = env::var_os("WASMER_BINFMT_MISC_PREOPEN").map(Into::into).unwrap_or(PathBuf::from("."));
		let mut wasi_env = WasiState::new(original_executable.to_string_lossy())
			.args(argv.iter().map(|arg| arg.as_bytes()))
			.envs(env::vars_os().map(|(k,v)| (k.into_vec(), v.into_vec())))
			.preopen_dir(preopen)?
			.finalize()?;
		let import_object = wasi_env.import_object(&module)?;

		let instance = Instance::new(&module, &import_object)?;
		let start = instance.exports.get_function("_start")?;
		let res = start.call(&[]);
		if let Err(Ok(WasiError::Exit(code))) = res.map_err(|e| e.downcast::<WasiError>()) {
			exit(code as _);
		}
	}

	Ok(())
}
