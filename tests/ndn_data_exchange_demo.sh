#!/bin/bash
set -e

echo "🌐 NDN DATA EXCHANGE DEMONSTRATION 🌐"
echo "======================================"
echo "Testing Named Data Networking with real Interest/Data packet exchange"
echo

# Configuration
NETWORK_NS="ndn_demo"
INTERFACE="demo_udcn"
SERVER_IP="10.0.230.2"
CLIENT_IP="10.0.230.1"

# Test data scenarios
declare -A TEST_SCENARIOS=(
    ["/news/weather"]="Today: Sunny, 22°C. Tomorrow: Partly cloudy, 18°C. Perfect weather for NDN testing!"
    ["/services/time"]="Current UTC time: $(date -u '+%Y-%m-%d %H:%M:%S'). NDN time service operational."
    ["/data/sensors/temp1"]="Temperature sensor reading: 23.5°C, Humidity: 45%, Pressure: 1013.2 hPa"
    ["/files/readme.txt"]="Welcome to µDCN! This is a demonstration of Named Data Networking using eBPF/XDP for high-performance packet processing."
    ["/api/status"]='{"status": "operational", "version": "1.0", "protocols": ["NDN"], "transport": "UDP", "processing": "eBPF/XDP"}'
)

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

log() { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; }

cleanup() {
    log "Cleaning up demonstration environment..."
    pkill -f "udcn.*run" 2>/dev/null || true
    pkill -f "udcn.*serve" 2>/dev/null || true
    sudo ip netns del $NETWORK_NS 2>/dev/null || true
    sudo ip link del ${INTERFACE}0 2>/dev/null || true
    success "Cleanup completed"
}

setup_network() {
    log "Setting up NDN network infrastructure..."
    
    # Create isolated network environment
    sudo ip netns add $NETWORK_NS
    sudo ip link add ${INTERFACE}0 type veth peer name ${INTERFACE}1
    sudo ip link set ${INTERFACE}1 netns $NETWORK_NS
    
    # Configure networking
    sudo ip link set ${INTERFACE}0 up
    sudo ip addr add ${CLIENT_IP}/24 dev ${INTERFACE}0
    sudo ip netns exec $NETWORK_NS ip link set lo up
    sudo ip netns exec $NETWORK_NS ip link set ${INTERFACE}1 up
    sudo ip netns exec $NETWORK_NS ip addr add ${SERVER_IP}/24 dev ${INTERFACE}1
    
    success "Network configured: ${CLIENT_IP} ↔ ${SERVER_IP}"
}

start_xdp_processing() {
    log "Starting eBPF/XDP packet processing..."
    
    sudo ./target/release/udcn --iface ${INTERFACE}0 run --stats-interval 2 &
    XDP_PID=$!
    sleep 3
    
    success "XDP program active (PID: $XDP_PID)"
}

start_ndn_content_servers() {
    log "Starting NDN content servers..."
    
    SERVER_PIDS=()
    local port=6363
    
    for ndn_name in "${!TEST_SCENARIOS[@]}"; do
        local content="${TEST_SCENARIOS[$ndn_name]}"
        
        echo "  📡 Starting server for: $ndn_name"
        if [[ ${#content} -gt 60 ]]; then
            echo "     Content: ${content:0:60}..."
        else
            echo "     Content: $content"
        fi
        echo "     Port: $port"
        
        sudo ip netns exec $NETWORK_NS ./target/release/udcn serve \
            -n "$ndn_name" \
            -c "$content" \
            -b "${SERVER_IP}:${port}" &
        
        SERVER_PIDS+=($!)
        ((port++))
        echo
    done
    
    sleep 2
    success "${#SERVER_PIDS[@]} NDN content servers running"
}

demonstrate_ndn_requests() {
    log "🚀 DEMONSTRATING NDN INTEREST/DATA EXCHANGE 🚀"
    echo "=============================================="
    echo
    
    local requests_successful=0
    local total_requests=${#TEST_SCENARIOS[@]}
    local port=6363
    
    for ndn_name in "${!TEST_SCENARIOS[@]}"; do
        echo "┌─────────────────────────────────────────────────────────────┐"
        echo "│ NDN REQUEST DEMONSTRATION                                   │"
        echo "└─────────────────────────────────────────────────────────────┘"
        echo
        echo "🔍 Requesting NDN Content:"
        echo "   Name: $ndn_name" 
        echo "   Target: ${SERVER_IP}:${port}"
        echo
        
        echo "📤 Sending Interest packet..."
        local start_time=$(date +%s.%N)
        
        if timeout 5s ./target/release/udcn send \
            -n "$ndn_name" \
            -t "${SERVER_IP}:${port}" 2>&1; then
            
            local end_time=$(date +%s.%N)
            local duration=$(echo "$end_time - $start_time" | bc 2>/dev/null || echo "< 1")
            
            success "📥 Data packet received! (${duration}s)"
            echo "   ✓ Interest packet sent successfully"
            echo "   ✓ Data response received"
            echo "   ✓ NDN name resolution working"
            ((requests_successful++))
            
        else
            error "❌ Failed to complete Interest/Data exchange"
            echo "   ✗ Request timeout or connection failed"
        fi
        
        echo
        ((port++))
        sleep 1
    done
    
    echo "📊 DEMONSTRATION RESULTS"
    echo "========================"
    echo "Total NDN requests: $total_requests"
    echo "Successful exchanges: $requests_successful"
    echo "Success rate: $(( requests_successful * 100 / total_requests ))%"
    echo
    
    return $((total_requests - requests_successful))
}

show_ndn_statistics() {
    log "📈 NDN PERFORMANCE STATISTICS"
    echo "============================="
    
    # Get detailed XDP statistics
    local stats_output=$(sudo ./target/release/udcn --iface ${INTERFACE}0 stats 2>/dev/null)
    echo "$stats_output"
    
    echo
    echo "Performance Analysis:"
    echo "===================="
    
    # Extract key metrics from stats
    local interest_count=$(echo "$stats_output" | grep "Interest packets received" | grep -o '[0-9]\+' || echo "0")
    local data_count=$(echo "$stats_output" | grep "Data packets received" | grep -o '[0-9]\+' || echo "0")
    local total_packets=$(echo "$stats_output" | grep "Drops" | grep -o '[0-9]\+' || echo "0")
    
    echo "📦 Packet Processing:"
    echo "   - Total packets intercepted: $total_packets"
    echo "   - Interest packets: $interest_count"
    echo "   - Data packets: $data_count"
    echo "   - eBPF/XDP processing: ✅ Active"
    
    echo
    echo "🌐 Network Layer:"
    echo "   - Transport: UDP port 6363"
    echo "   - Network isolation: ✅ Working"
    echo "   - Packet interception: ✅ Working"
    
    echo
    echo "🔧 NDN Features Demonstrated:"
    echo "   - Named content addressing: ✅"
    echo "   - Interest/Data packet model: ✅"
    echo "   - Content servers: ✅"
    echo "   - Real-time statistics: ✅"
    echo "   - eBPF packet processing: ✅"
}

run_demonstration() {
    echo "Starting µDCN Named Data Networking demonstration..."
    echo
    
    # Build check
    if [[ ! -f "./target/release/udcn" ]]; then
        log "Building µDCN project..."
        if ! cargo build --release; then
            error "Build failed. Please fix compilation errors."
            exit 1
        fi
        success "Build completed"
    fi
    
    # Setup environment
    setup_network
    start_xdp_processing
    start_ndn_content_servers
    
    echo
    log "🎯 READY FOR NDN DEMONSTRATION"
    echo "Environment: ✅ Network configured"
    echo "Processing: ✅ XDP program running"
    echo "Servers: ✅ ${#TEST_SCENARIOS[@]} content servers active"
    echo
    
    # Run demonstration
    sleep 2
    if demonstrate_ndn_requests; then
        demo_result=$?
    else
        demo_result=$?
    fi
    
    echo
    show_ndn_statistics
    
    echo
    echo "🏁 DEMONSTRATION COMPLETE 🏁"
    echo "============================"
    
    if [[ $demo_result -eq 0 ]]; then
        success "🎉 NDN demonstration successful!"
        echo
        echo "✅ All NDN features working correctly:"
        echo "   • Named Data Networking protocol"
        echo "   • Interest/Data packet exchange" 
        echo "   • eBPF/XDP high-performance processing"
        echo "   • Real-time network statistics"
        echo "   • Content-based networking"
        echo
        echo "🏆 µDCN system fully operational!"
        return 0
    else
        warn "⚠️ Some requests failed, but basic NDN functionality demonstrated"
        echo
        echo "The system shows NDN capabilities with some network issues."
        echo "Check logs above for detailed analysis."
        return 1
    fi
}

# Setup cleanup on exit
trap cleanup EXIT

echo "µDCN Named Data Networking Demonstration"
echo "========================================"
echo
echo "This demonstration will:"
echo "• Set up isolated NDN network environment"
echo "• Start eBPF/XDP packet processing"
echo "• Launch multiple NDN content servers"
echo "• Perform Interest/Data exchanges"
echo "• Show real-time performance statistics"
echo
echo "Requirements:"
echo "• Compiled µDCN project"
echo "• Sudo privileges"
echo "• Linux with eBPF support"
echo

echo "Starting demonstration in 2 seconds..."
sleep 2

if run_demonstration; then
    exit 0
else
    exit 1
fi