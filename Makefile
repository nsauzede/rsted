
all: test

test:
	cargo --locked check
	cargo --locked llvm-cov
	cargo --locked clippy -- -D warnings
	cargo --locked fmt -- --check
	cargo --locked audit
	cargo --locked build

clean:
	cargo --locked clean
