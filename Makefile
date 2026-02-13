.PHONY: serve

serve:
	python3 -m http.server --directory docs 8000

install:
	cargo install --path .
