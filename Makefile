.DEFAULT_GOAL := help
.PHONY: help build check test clippy fmt install clean release

help: ## Show this help message
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

build: ## Build the project
	cargo build

check: ## Run cargo check
	cargo check

test: ## Run tests
	cargo test

clippy: ## Run clippy lints
	cargo clippy -- -D warnings

fmt: ## Format code
	cargo fmt --check

fmt-fix: ## Fix formatting
	cargo fmt

install: ## Install envy
	@echo "Installing envy..."
	@cargo install --path .

clean: ## Clean build artifacts
	cargo clean

release: ## Build release version
	cargo build --release

ci: check test clippy fmt ## Run all CI checks