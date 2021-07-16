# Experiment

Set up either from the inside of docker
```
docker build -t register-wasmer-binfmt .
docker run --rm --privileged -v /proc/sys/fs/binfmt_misc/:/binfmt_misc/ register-wasmer-binfmt
```

Or normally (untested)
```
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl
sudo target/x86_64-unknown-linux-musl/release/target/x86_64-unknown-linux-musl/release/wasmer-static /proc/sys/fs/binfmt_misc/
```

Confirm that the interpreter has been installed
```
cat /proc/sys/fs/binfmt_misc/wasm
```
and enjoy running `wasm` binaries anywhere on your machine (with terrible start-up latency, because caching is too complicated for this experiment).

Crazy idea: Can I have a wasm-based busybox run httpd to spawn a wasm-based cgi, in docker?
