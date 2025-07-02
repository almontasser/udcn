# µDCN Test Suite

This directory contains comprehensive tests for the µDCN (micro Named Data Networking) implementation.

## Test Structure

```
tests/
├── unit/               # Unit tests for core functionality
│   ├── packet_tests.rs    # NDN packet structure tests
│   ├── hash_tests.rs      # Name hashing function tests
│   └── serialization_tests.rs # Packet serialization tests
├── integration/        # End-to-end integration tests
│   └── basic_ndn_test.rs  # NDN Interest/Data exchange tests
├── performance/        # Performance and stress tests
│   └── stress_test.rs     # High-throughput and concurrent tests
└── README.md          # This file
```

## Running Tests

### Quick Unit Tests
```bash
make test-unit
```

### Integration Tests (requires sudo)
```bash
sudo make test-integration
```

### Performance Tests (requires sudo)
```bash
sudo make test-performance
```

### All Tests
```bash
sudo make test
```

## Test Categories

### Unit Tests
- **Packet Tests**: Verify NDN packet structures (Interest, Data, TLV headers)
- **Hash Tests**: Test name hashing consistency and collision resistance
- **Serialization Tests**: Test packet serialization/deserialization

### Integration Tests
- **Basic NDN Test**: End-to-end Interest/Data exchange
- **XDP Loading**: Verify eBPF program loads without errors
- **Multiple Interests**: Test handling of sequential requests

### Performance Tests
- **High Throughput**: Test rapid Interest sending
- **Concurrent Clients**: Test multiple simultaneous clients
- **Packet Processing**: Measure XDP processing performance

## Requirements

### Software Dependencies
- Rust toolchain
- cargo
- sudo privileges (for network namespace creation)

### System Requirements
- Linux kernel with eBPF support
- XDP support
- Network namespace support (`ip netns`)

## Test Network Setup

The tests automatically create isolated network environments using:
- Network namespaces (`ip netns`)
- Virtual Ethernet pairs (`veth`)
- Custom IP addresses (10.0.x.x/24 ranges)

All test networks are automatically cleaned up after test completion.

## Troubleshooting

### Permission Errors
Ensure you're running integration and performance tests with sudo:
```bash
sudo make test-integration
```

### Network Cleanup
If tests fail and leave network artifacts, clean up manually:
```bash
make clean
```

### Build Errors
Ensure the project builds successfully first:
```bash
make build
```

## Test Output

Tests provide detailed output including:
- Packet processing statistics
- Performance metrics (throughput, latency)
- XDP program status
- Network configuration details

## Adding New Tests

### Unit Tests
Add new test functions to existing files in `tests/unit/` or create new test files.

### Integration Tests
Create new test functions in `tests/integration/` that use the network setup helpers.

### Performance Tests
Add new performance scenarios to `tests/performance/stress_test.rs`.

## Continuous Integration

Tests are designed to be run in CI environments with appropriate permissions for network namespace creation.