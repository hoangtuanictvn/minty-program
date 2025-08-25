# Bonding Curve Program Makefile

.PHONY: help build test deploy clean client-install client-test all

# Default target
help:
	@echo "🚀 Bonding Curve Program Commands"
	@echo ""
	@echo "Build Commands:"
	@echo "  build          - Build the Rust program"
	@echo "  build-sbf      - Build for Solana BPF"
	@echo ""
	@echo "Test Commands:"
	@echo "  test           - Run Rust unit tests"
	@echo "  test-client    - Run TypeScript client tests"
	@echo "  test-all       - Run all tests"
	@echo ""
	@echo "Deploy Commands:"
	@echo "  deploy-devnet  - Deploy to devnet"
	@echo "  deploy-mainnet - Deploy to mainnet"
	@echo ""
	@echo "Client Commands:"
	@echo "  client-install - Install client dependencies"
	@echo "  client-build   - Build client TypeScript"
	@echo "  client-test    - Run client tests"
	@echo ""
	@echo "Utility Commands:"
	@echo "  clean          - Clean build artifacts"
	@echo "  format         - Format Rust code"
	@echo "  clippy         - Run Rust linter"
	@echo "  all            - Build, test, and prepare for deployment"

# Build commands
build:
	@echo "🔨 Building Rust program..."
	cargo build

build-sbf:
	@echo "🔨 Building for Solana BPF..."
	cargo build-sbf

# Test commands
test:
	@echo "🧪 Running Rust unit tests..."
	cargo test

test-client:
	@echo "🧪 Running client tests..."
	@chmod +x scripts/test.sh
	./scripts/test.sh client

test-all:
	@echo "🧪 Running all tests..."
	@chmod +x scripts/test.sh
	./scripts/test.sh all

# Deploy commands
deploy-devnet:
	@echo "🚀 Deploying to devnet..."
	@chmod +x scripts/deploy.sh
	./scripts/deploy.sh devnet

deploy-mainnet:
	@echo "🚀 Deploying to mainnet..."
	@chmod +x scripts/deploy.sh
	./scripts/deploy.sh mainnet-beta

# Client commands
client-install:
	@echo "📦 Installing client dependencies..."
	cd client && npm install

client-build:
	@echo "🔨 Building client TypeScript..."
	cd client && npm run build

client-test: client-install
	@echo "🧪 Running client tests..."
	cd client && npm run test

# Utility commands
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean
	rm -rf target/
	rm -rf client/dist/
	rm -rf client/node_modules/
	rm -f deployment.json
	rm -f client/payer-keypair.json

format:
	@echo "✨ Formatting Rust code..."
	cargo fmt

clippy:
	@echo "📎 Running Rust linter..."
	cargo clippy -- -D warnings

# Comprehensive build and test
all: format clippy build build-sbf test client-install client-build
	@echo "✅ All build and test steps completed!"
	@echo ""
	@echo "Next steps:"
	@echo "1. Run 'make deploy-devnet' to deploy to devnet"
	@echo "2. Run 'make client-test' to test the client"
	@echo "3. Fund the generated test account if needed"

# Development workflow
dev: format build test
	@echo "🔄 Development build completed"

# Quick test cycle
quick-test: build test
	@echo "⚡ Quick test cycle completed"

# Setup development environment
setup:
	@echo "🛠️  Setting up development environment..."
	@echo "Installing Rust dependencies..."
	cargo check
	@echo "Installing client dependencies..."
	cd client && npm install
	@echo "✅ Development environment ready!"

# Check program size
check-size: build-sbf
	@echo "📏 Program size:"
	@ls -lh target/deploy/x_token.so

# Verify deployment
verify:
	@echo "🔍 Verifying deployment..."
	@if [ -f deployment.json ]; then \
		echo "📋 Deployment found:"; \
		cat deployment.json | jq .; \
	else \
		echo "❌ No deployment found. Run 'make deploy-devnet' first."; \
	fi
