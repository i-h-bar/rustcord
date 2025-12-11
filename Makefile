setup:
	@rustup target add aarch64-unknown-linux-gnu

build_pi:
	@cargo build --release --target aarch64-unknown-linux-gnu

lint:
	@cargo fmt
	@cargo clippy

mega_lint:
	@cargo fmt
	@cargo clippy -- -W clippy::pedantic

coverage:
	@cargo llvm-cov --all-features --no-cfg-coverage

coverage-html:
	@cargo llvm-cov --all-features --no-cfg-coverage --html
	@echo "Coverage report generated at target/llvm-cov/html/index.html"

coverage-open:
	@cargo llvm-cov --all-features --no-cfg-coverage --open
