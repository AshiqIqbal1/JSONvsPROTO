#!/bin/bash
# Simulate slow network on loopback for ports 8001 and 8002
# Defaults: 5Mbit bandwidth, 50ms delay (simulates ~cross-region API call)
BW=${1:-5Mbit}
DELAY=${2:-50}

echo "Applying: ${BW} bandwidth, ${DELAY}ms delay to localhost:8001 and localhost:8002"

# Create dummynet pipe
sudo dnctl pipe 1 config bw $BW delay $DELAY

# Write pf rules to anchor
sudo pfctl -E 2>/dev/null
cat <<EOF | sudo pfctl -a benchmark -f -
dummynet out proto tcp from any to 127.0.0.1 port 8001 pipe 1
dummynet out proto tcp from any to 127.0.0.1 port 8002 pipe 1
EOF

echo "Network throttle active. Run benchmark, then: ./network_reset.sh"
