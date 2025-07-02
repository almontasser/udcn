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

# Run NDN data exchange demonstration (requires sudo)
test-ndn-demo:
	@echo "Running NDN data exchange demonstration..."
	@echo "Note: This requires sudo privileges for network setup"
	sudo ./tests/ndn_data_exchange_demo.sh

# Run file transfer demonstration (requires sudo)
test-file-transfer:
	@echo "Running NDN file transfer test..."
	@echo "Note: This requires sudo privileges for network setup"
	sudo ./tests/simple_file_transfer_test.sh

# Run comprehensive file exchange test (requires sudo)
test-file-exchange:
	@echo "Running comprehensive file exchange test..."
	@echo "Note: This requires sudo privileges and may take several minutes"
	sudo ./tests/ndn_file_exchange_test.sh

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
	@echo "  build               - Build the project"
	@echo "  test                - Run all tests"
	@echo "  test-unit           - Run unit tests only"
	@echo "  test-integration    - Run integration tests (requires sudo)"
	@echo "  test-performance    - Run performance tests (requires sudo)"
	@echo "  test-ndn-demo       - Run NDN data exchange demo (requires sudo)"
	@echo "  test-file-transfer  - Run file transfer test (requires sudo)"
	@echo "  test-file-exchange  - Run comprehensive file exchange (requires sudo)"
	@echo "  test-quick          - Run quick unit tests"
	@echo "  clean               - Clean build artifacts and test networks"
	@echo "  help                - Show this help"
	@echo ""
	@echo "Examples:"
	@echo "  make build              # Build the project"
	@echo "  make test-unit          # Run unit tests"
	@echo "  sudo make test-ndn-demo # Run NDN demonstration"
	@echo "  sudo make test          # Run all tests (requires sudo)"