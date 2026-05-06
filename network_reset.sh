#!/bin/bash
# Remove network simulation
sudo pfctl -a benchmark -F all 2>/dev/null
sudo dnctl -q flush
sudo pfctl -d 2>/dev/null
echo "Network throttle removed."
