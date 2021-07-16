use std::env;
use std::fs;
use std::path::PathBuf;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::os::unix::ffi::OsStrExt;

fn main() {
	let mut argv = env::args_os();
	argv.next();
	let wasmer = PathBuf::from(argv.next()
		.expect("First argument must be path to wasmer-static"));
	if !wasmer.exists() {
		panic!("{} does not exist", wasmer.to_string_lossy());
	}
	let wasmer = fs::canonicalize(&wasmer)
		.expect(&format!("Couldn't get absolute path for {}", wasmer.to_string_lossy()));
	let binfmt = PathBuf::from(argv.next()
		.expect("Second argument must be path to binfmt_misc fs"));
	if !binfmt.exists() {
		panic!("{} does not exist", binfmt.to_string_lossy());
	}
	let existing = binfmt.join("wasm");
	if existing.exists() {
		let mut existing = OpenOptions::new()
			.write(true)
			.open(existing).expect("Open existing entry to remove");
		existing.write_all(b"-1").expect("Couldn't write unregister request");
	}
	let register = binfmt.join("register");
	let mut register = OpenOptions::new()
		.write(true)
		.open(register).expect("Open existing entry to remove");
	let spec = [b":wasm:M::\\x00asm\\x01\\x00\\x00::".as_ref(), wasmer.as_os_str().as_bytes(), b":PFC"];
	register.write_all(&spec.concat()).expect("Couldn't register binfmt");
}
