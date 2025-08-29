#!/bin/bash

# Colors
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m'

echo -e "${YELLOW}Stopping all minimal multiplayer processes...${NC}"

# Kill any cargo run processes for this project
pkill -f "minimal_server"
pkill -f "minimal_client"
pkill -f "cargo run"

# Also try to kill any processes using the server port
lsof -ti:5000 | xargs -r kill -9

echo -e "${YELLOW}All processes stopped${NC}"