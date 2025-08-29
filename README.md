# Minimal Multiplayer Setup with Lightyear

A super minimal multiplayer game setup using Bevy and Lightyear for networking. The client sends ping messages every 2 seconds, and the server responds with pong messages.

## Project Structure

- `shared/` - Common protocol definition (messages, channels, constants)
- `server/` - Headless server that listens for clients and responds to pings
- `client/` - Client with Bevy rendering that connects and sends pings

## Quick Start

### Run Both (Recommended)
```bash
./run.sh
```
This starts both server and client with colored prefixed output. Press Ctrl+C to stop both.

### Run Separately
```bash
# Terminal 1 - Server
cd server && cargo run

# Terminal 2 - Client  
cd client && cargo run
```

## How It Works

1. **Server** starts and listens on `localhost:5000`
2. **Client** connects automatically and spawns a 2D camera
3. **Client** sends `PingMessage("Hello from client!")` every 2 seconds
4. **Server** receives pings and responds with `PingMessage("Pong! Got: Hello from client!")`
5. **Client** logs received pong messages

## Protocol

- **Message**: `PingMessage(String)` - Simple string message
- **Channel**: `Channel1` - Reliable, ordered delivery
- **Direction**: Bidirectional (client ↔ server)
- **Transport**: UDP with netcode.io security

## Key Files

- `shared/src/lib.rs` - Protocol definition
- `server/src/main.rs` - Server implementation
- `client/src/main.rs` - Client implementation
- `run.sh` - Convenience script to run both

## Dependencies

- Bevy 0.16.1
- Lightyear 0.23.0
- Serde for serialization