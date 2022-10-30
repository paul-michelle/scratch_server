.PHONY: clean serve
default: clean

clean:
	cargo fmt && cargo clippy

serve:
	fuser -k 7878/tcp || true && cargo run

test: clean
	cargo test