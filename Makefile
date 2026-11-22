TARGET ?= $(shell rustc -vV | sed -n 's/^host: //p')
VERSION := $(shell sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml)
BINARY = target/$(TARGET)/release/declutter
PACKAGE = declutter-$(VERSION)-$(TARGET)

.PHONY: build install package test
build:
	cargo build --locked --release --target-dir target --target "$(TARGET)" --bin declutter

install: build
	./scripts/install.sh "$(BINARY)"

package: build
	@mkdir -p dist
	@set -eu; stage=$$(mktemp -d "dist/.stage.XXXXXX"); \
	trap 'rm -rf "$$stage"' EXIT; \
	mkdir "$$stage/$(PACKAGE)"; \
	cp "$(BINARY)" scripts/install.sh README.md LICENSE "$$stage/$(PACKAGE)/"; \
	tar -czf "$$stage/archive.tar.gz" -C "$$stage" "$(PACKAGE)"; \
	mv "$$stage/archive.tar.gz" "dist/$(PACKAGE).tar.gz"; \
	echo "dist/$(PACKAGE).tar.gz"

test:
	cargo test --locked
