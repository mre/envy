.DEFAULT_GOAL := help

.PHONY: help
help: ## Show this help message
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

.PHONY: build
build: ## Build the project
	cargo build

.PHONY: check
check: ## Run cargo check
	cargo check

.PHONY: test
test: ## Run tests
	cargo test -- --test-threads=1

.PHONY: test-integration
test-integration: ## Run integration tests only
	cargo test --test cli -- --test-threads=1

.PHONY: clippy
clippy: ## Run clippy lints
	cargo clippy -- -D warnings

.PHONY: fmt
fmt: ## Format code
	cargo fmt --check

.PHONY: fmt-fix
fmt-fix: ## Fix formatting
	cargo fmt

.PHONY: install
install: ## Install envy
	@echo "Installing envy..."
	@cargo install --path .

.PHONY: clean
clean: ## Clean build artifacts
	cargo clean

.PHONY: release
release: ## Build release version
	cargo build --release

.PHONY: ci
ci: check test clippy fmt ## Run all CI checks