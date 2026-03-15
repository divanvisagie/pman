.PHONY: test install serve

test:
	cargo test

install:
	cargo install --path .

serve:
	python3 -m http.server --directory docs 8000
