# Set the Rust compiler and options for release mode
RUSTC=cargo rustc
RUSTFLAGS=--release

.PHONY: install-dev-commands
install-dev-commands:
	cargo install cargo-tarpaulin

# Build the JMESPath Community crate and CLI
.PHONY: build
build:
	cd jmespath/ && cargo fmt --all && cargo build --features preserve_order
	cd jp/ && cargo build

.PHONY: release
release:
	cd jmespath/ && $(RUSTC) $(RUSTFLAGS) --lib
	cd jp/ && $(RUSTC) $(RUSTFLAGS) --bin jp


# Run the JMESPath CLI
.PHONY: run
run: build
	cd jp/ && cargo run --bin jp

# Publish documentation for the JMESPath Community
.PHONY: doc
doc:
	cd jmespath/ && cargo doc

# Run tests for the JMESPath Community crate
.PHONY: compliance
compliance: build
	git submodule update --init
	cd tests/ && \
	cargo run -- -f

.PHONY: test
test: build
	cargo test --features preserve_order

.PHONY: coverage
coverage: install-dev-commands test
	cd jmespath/ && \
	cargo tarpaulin --features preserve_order --out Html --output-dir ./coverage-report/ \

# Clean the build artifacts
.PHONY: clean
clean:
	cd jp/ && cargo clean
	cd jmespath/ && cargo clean

clobber: clean
	rm -rf coverage-report/
