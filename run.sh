#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# Store PIDs for cleanup
SERVER_PID=""
CLIENT_PID=""

# Function to handle cleanup on script exit
cleanup() {
    echo -e "\n${YELLOW}Shutting down server and client...${NC}"
    
    # Kill client first
    if [ ! -z "$CLIENT_PID" ]; then
        kill -TERM $CLIENT_PID 2>/dev/null
        sleep 1
        kill -KILL $CLIENT_PID 2>/dev/null
    fi
    
    # Kill server
    if [ ! -z "$SERVER_PID" ]; then
        kill -TERM $SERVER_PID 2>/dev/null
        sleep 1
        kill -KILL $SERVER_PID 2>/dev/null
    fi
    
    # Kill any remaining cargo processes
    pkill -f "cargo run" 2>/dev/null
    
    echo -e "${YELLOW}Cleanup complete${NC}"
    exit 0
}

# Set up signal handlers
trap cleanup SIGINT SIGTERM EXIT

echo -e "${BLUE}Starting minimal multiplayer demo...${NC}"
echo -e "${BLUE}Press Ctrl+C to stop both server and client${NC}"
echo

# Start server in background and prefix its output
(cd server && exec cargo run 2>&1 | sed "s/^/$(printf "${RED}[Server]${NC} ")/") &
SERVER_PID=$!

# Give server a moment to start
sleep 2

# Start client in background and prefix its output  
(cd client && exec cargo run 2>&1 | sed "s/^/$(printf "${GREEN}[Client]${NC} ")/") &
CLIENT_PID=$!

echo -e "${YELLOW}Server PID: $SERVER_PID${NC}"
echo -e "${YELLOW}Client PID: $CLIENT_PID${NC}"
echo

# Wait for either process to exit
while kill -0 $SERVER_PID 2>/dev/null && kill -0 $CLIENT_PID 2>/dev/null; do
    sleep 1
done

# If we get here, one process exited, so cleanup
cleanup
