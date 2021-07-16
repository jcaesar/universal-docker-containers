FROM ekidd/rust-musl-builder as build
COPY . .
RUN cargo build --release --locked

FROM scratch
COPY --from=build /home/rust/src/target/x86_64-unknown-linux-musl/release/register /home/rust/src/target/x86_64-unknown-linux-musl/release/wasmer-static /
ENTRYPOINT ["/register", "/wasmer-static", "/binfmt_misc/"]
