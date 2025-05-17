setup:
	@rustup target add aarch64-unknown-linux-gnu

build_pi:
	@cargo build --release --target aarch64-unknown-linux-gnu


lint:
	@cargo fmt
	@cargo clippy -- -D warnings