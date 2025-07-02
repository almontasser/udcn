#!/bin/bash
set -euo pipefail

IFACE=${1:-udcn0}
IP=10.0.100.1
INTERVAL=2

cleanup() {
    echo "Cleaning up" >&2
    sudo ip link del "$IFACE" 2>/dev/null || true
    if [ -n "${DAEMON_PID:-}" ]; then sudo kill "$DAEMON_PID" 2>/dev/null || true; fi
    if [ -n "${SERVER_PID:-}" ]; then kill "$SERVER_PID" 2>/dev/null || true; fi
}
trap cleanup EXIT

echo "Building µDCN..."
cargo build --release

echo "Setting up dummy interface $IFACE"
sudo ip link add name "$IFACE" type dummy 2>/dev/null || true
sudo ip link set "$IFACE" up
sudo ip addr add "$IP/24" dev "$IFACE" 2>/dev/null || true

echo "Starting XDP daemon..."
sudo ./target/release/udcn --iface "$IFACE" run --stats-interval "$INTERVAL" &
DAEMON_PID=$!
sleep 2

echo "Starting data server..."
./target/release/udcn serve -n "/test/data" -c "Hello from µDCN" -b "$IP:6363" &
SERVER_PID=$!
sleep 1

echo "Sending interest packet"
./target/release/udcn send -n "/test/data" -t "$IP:6363"

sleep "$((INTERVAL+1))"

echo "\nCollected statistics:"
sudo ./target/release/udcn stats
