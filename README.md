# µDCN - Minimal Data-Centric Networking with eBPF/XDP

A minimal but functional NDN (Named Data Networking) implementation using eBPF/XDP for high-performance packet processing in the Linux kernel.

## Features

- **eBPF/XDP Integration**: Fast packet processing in kernel space
- **Basic NDN Packets**: Interest and Data packets with TLV encoding
- **Content Store**: LRU cache for data packets in eBPF maps
- **Pending Interest Table (PIT)**: Track pending interests
- **Statistics Collection**: Real-time performance metrics
- **CLI Interface**: Easy-to-use command line tools

## Architecture

```
┌─────────────────┐    ┌──────────────────┐
│   User Space    │    │   Kernel Space   │
│                 │    │                  │
│ ┌─────────────┐ │    │ ┌──────────────┐ │
│ │ CLI Tool    │ │    │ │ XDP Program  │ │
│ │ Statistics  │ │◄──►│ │ Packet Proc. │ │
│ │ Map Manager │ │    │ │ PIT/CS/Stats │ │
│ └─────────────┘ │    │ └──────────────┘ │
└─────────────────┘    └──────────────────┘
```

## Prerequisites

1. stable rust toolchains: `rustup toolchain install stable`
1. nightly rust toolchains: `rustup toolchain install nightly --component rust-src`
1. bpf-linker: `cargo install bpf-linker` (`--no-default-features` on macOS)
1. Linux kernel 5.4+ with XDP support
1. Root privileges for XDP program loading

## Quick Start

### Build

```bash
cargo build --release
```

### Setup Dedicated NDN Interface

```bash
sudo ip link add name udcn0 type dummy
sudo ip link set udcn0 up
sudo ip addr add 10.0.100.1/24 dev udcn0
```

### Run XDP Daemon (requires root)

```bash
sudo ./target/release/udcn run --stats-interval 5
```

### Send Interest Packet

```bash
./target/release/udcn send -n "/test/data" -t "10.0.100.1:6363"
```

### Serve Data

```bash
./target/release/udcn serve -n "/test/data" -c "Hello World!" -b "10.0.100.1:6363"
```

### View Statistics

```bash
./target/release/udcn stats
```

## Usage Examples

### 1. Basic Interest/Data Exchange

Terminal 1 (start data server):
```bash
./target/release/udcn serve -n "/video/stream1" -c "Video data content" -b "127.0.0.1:6363"
```

Terminal 2 (send interest):
```bash
./target/release/udcn send -n "/video/stream1" -t "127.0.0.1:6363"
```

### 2. XDP Performance Mode

Terminal 1 (start XDP daemon with stats):
```bash
sudo ./target/release/udcn run --stats-interval 5
```

Terminal 2 (generate traffic):
```bash
for i in {1..100}; do
  ./target/release/udcn send -n "/test/data$i" -t "127.0.0.1:6363"
done
```

Terminal 3 (view statistics):
```bash
./target/release/udcn stats
```

## Testing

Run the automated test suite using the helper script. This script builds the
project, sets up a temporary test interface, runs a quick Interest/Data
exchange, and prints the resulting statistics:

```bash
./test_udcn.sh
```

Run performance benchmarks:

```bash
cargo run --example benchmark
```

## Components

### Core Libraries

- **udcn-common**: Shared packet structures and parsing logic
- **udcn-ebpf**: XDP program for kernel-space packet processing  
- **udcn**: User-space CLI and management tools

### Key Features

1. **Packet Processing**: Recognizes NDN Interest/Data packets
2. **Content Store**: LRU cache with configurable size (512 entries)
3. **PIT Management**: Hash map for pending interests (1024 entries)
4. **Statistics**: Real-time metrics collection
5. **Performance**: Line-rate processing with eBPF/XDP

## Cross-compiling on macOS

Cross compilation should work on both Intel and Apple Silicon Macs.

```shell
CC=${ARCH}-linux-musl-gcc cargo build --package udcn --release \
  --target=${ARCH}-unknown-linux-musl \
  --config=target.${ARCH}-unknown-linux-musl.linker=\"${ARCH}-linux-musl-gcc\"
```
The cross-compiled program `target/${ARCH}-unknown-linux-musl/release/udcn` can be
copied to a Linux server or VM and run there.

## License

With the exception of eBPF code, udcn is distributed under the terms
of either the [MIT license] or the [Apache License] (version 2.0), at your
option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

### eBPF

All eBPF code is distributed under either the terms of the
[GNU General Public License, Version 2] or the [MIT license], at your
option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you, as defined in the GPL-2 license, shall be
dual licensed as above, without any additional terms or conditions.

[Apache license]: LICENSE-APACHE
[MIT license]: LICENSE-MIT
[GNU General Public License, Version 2]: LICENSE-GPL2
