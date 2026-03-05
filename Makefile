TLS_CERT ?= $(HOME)/.config/plan/tls/plan-local-cert.pem
TLS_KEY  ?= $(HOME)/.config/plan/tls/plan-local-key.pem

.PHONY: serve install mcp

serve:
	python3 -m http.server --directory docs 8000

install:
	cargo install --path .

mcp:
	cargo run -- mcp --tls-cert $(TLS_CERT) --tls-key $(TLS_KEY) --port 3109
