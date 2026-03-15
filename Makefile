.PHONY: test install serve

PYTHON := /home/divan/.local/share/pipx/venvs/pman-mcp/bin/python

test:
	cargo test
	$(PYTHON) -m unittest discover -s tests_python -v

install:
	cargo install --path .
	pipx install --force .

serve:
	python3 -m http.server --directory docs 8000
