# µDCN Makefile
.PHONY: all build test test-unit test-integration test-performance clean help

# Default target
all: build

# Build the project
build:
	@echo "Building µDCN..."
	cargo build --release
	@echo "✅ Build completed successfully!"

# Run all tests
test: test-unit test-integration test-performance

# Run unit tests only
test-unit:
	@echo "Running unit tests..."
	cargo test -p udcn-common --features std

# Run integration tests (requires sudo for network setup)
test-integration:
	@echo "Running integration tests..."
	@echo "Note: These tests require sudo privileges for network setup"
	cargo test integration_tests -- --ignored

# Run performance/stress tests (requires sudo)
test-performance:
	@echo "Running performance tests..."
	@echo "Note: These tests require sudo privileges and may take several minutes"
	cargo test performance_tests -- --ignored

# Run a quick test of basic functionality
test-quick:
	@echo "Running quick unit tests..."
	cargo test -p udcn-common --features std test_packet_structures
	cargo test -p udcn-common --features std test_hash_consistency

# Clean build artifacts
clean:
	cargo clean
	sudo ip netns list | grep -E "(test_ndn|perf_ndn)" | xargs -r -I {} sudo ip netns del {}
	sudo ip link list | grep -E "(test_udcn|perf_udcn)" | awk '{print $$2}' | sed 's/:$$//' | xargs -r -I {} sudo ip link del {}

# Show help
help:
	@echo "µDCN Test Suite"
	@echo "==============="
	@echo ""
	@echo "Available targets:"
	@echo "  build           - Build the project"
	@echo "  test            - Run all tests"
	@echo "  test-unit       - Run unit tests only"
	@echo "  test-integration- Run integration tests (requires sudo)"
	@echo "  test-performance- Run performance tests (requires sudo)"
	@echo "  test-quick      - Run quick unit tests"
	@echo "  clean           - Clean build artifacts and test networks"
	@echo "  help            - Show this help"
	@echo ""
	@echo "Examples:"
	@echo "  make build      # Build the project"
	@echo "  make test-unit  # Run unit tests"
	@echo "  sudo make test  # Run all tests (requires sudo)"