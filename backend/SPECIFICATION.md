# Strategy King Backend API Specification

## Overview

The Strategy King backend is an AdonisJS 6 application that provides authentication, matchmaking, and game server orchestration for a multiplayer strategy game. The backend manages player sessions, matchmaking queues, and dynamically spawns game server instances.

## Technology Stack

- **Framework**: AdonisJS 6 with TypeScript
- **Database**: SQLite with Lucid ORM
- **Authentication**: JWT tokens
- **WebSocket**: AdonisJS Transmit
- **Validation**: VineJS
- **Testing**: Japa test runner

## Database Schema

### Players Table
```sql
- id (primary key)
- username (unique, not null)
- password_hash (not null)
- created_at
- updated_at
```

### Matches Table
```sql
- id (primary key)
- player_ids (JSON array)
- status (enum: pending, active, completed)
- server_port (nullable)
- auth_token (nullable)
- created_at
- updated_at
```

### Matchmaking Queue Table
```sql
- id (primary key)
- player_id (foreign key to players)
- game_mode (default: '1v1')
- created_at
- updated_at
```

## API Endpoints

### Authentication

#### POST /auth/register
Register a new player account.

**Request Body:**
```json
{
  "username": "string",
  "password": "string"
}
```

**Response (201):**
```json
{
  "success": true,
  "message": "Player registered successfully",
  "data": {
    "player_id": 123,
    "username": "player_name",
    "token": "jwt_token_here"
  }
}
```

**Error (400):**
```json
{
  "success": false,
  "message": "Username already exists"
}
```

#### POST /auth/login
Authenticate an existing player.

**Request Body:**
```json
{
  "username": "string",
  "password": "string"
}
```

**Response (200):**
```json
{
  "success": true,
  "message": "Login successful",
  "data": {
    "player_id": 123,
    "username": "player_name",
    "token": "jwt_token_here"
  }
}
```

**Error (401):**
```json
{
  "success": false,
  "message": "Invalid credentials"
}
```

### Matchmaking

#### POST /matchmaking/join
Join the matchmaking queue.

**Headers:**
```
Authorization: Bearer <jwt_token>
```

**Request Body:**
```json
{
  "gameMode": "1v1",
  "playerId": 123
}
```

**Response (200):**
```json
{
  "success": true,
  "queue_position": 1,
  "estimated_wait_seconds": 15
}
```

**Error (400):**
```json
{
  "success": false,
  "message": "Player already in queue"
}
```

**Error (403):**
```json
{
  "success": false,
  "message": "Unauthorized"
}
```

#### POST /matchmaking/leave
Leave the matchmaking queue.

**Headers:**
```
Authorization: Bearer <jwt_token>
```

**Request Body:**
```json
{
  "playerId": 123
}
```

**Response (200):**
```json
{
  "success": true,
  "message": "Left matchmaking queue"
}
```

## WebSocket Communication

### Connection
WebSocket connections must be authenticated using JWT tokens passed as query parameters.

**Connection URL:**
```
ws://localhost:3333/ws?token=<jwt_token>
```

### Events

#### Queue Status Updates
Sent when player's queue position changes.

```json
{
  "type": "queue_status",
  "data": {
    "queue_position": 1,
    "estimated_wait_seconds": 10
  }
}
```

#### Match Found
Sent when a match is found and game server is ready.

```json
{
  "type": "match_found",
  "data": {
    "match_id": 456,
    "server_port": 5001,
    "auth_token": "game_server_jwt_token",
    "player_ids": [123, 124]
  }
}
```

#### Connection Error
Sent when authentication fails.

```json
{
  "type": "error",
  "data": {
    "message": "Authentication failed"
  }
}
```

## Game Server Management

### Port Allocation
- Ports are allocated sequentially starting from 5001
- Maximum 100 concurrent game servers (ports 5001-5100)
- Released ports are reused only when sequential allocation is exhausted

### Game Server Environment Variables
When spawning a Rust game server, the following environment variables are passed:

- `AUTH_TOKEN_SECRET`: JWT signing secret for token validation
- `MATCH_ID`: Unique identifier for the match
- `EXPECTED_PLAYERS`: Number of players expected to join
- `SERVER_PORT`: Port number for the game server to bind to

### JWT Game Tokens
Game server tokens contain:
```json
{
  "match_id": "match_12345",
  "player_ids": [123, 124],
  "exp": 1234567890
}
```

Token expiration: 15 minutes from creation.

## Error Handling

All API endpoints return consistent error responses:

```json
{
  "success": false,
  "message": "Description of the error"
}
```

HTTP Status Codes:
- `200`: Success
- `201`: Created successfully
- `400`: Bad Request (validation errors, business logic errors)
- `401`: Unauthorized (invalid credentials)
- `403`: Forbidden (insufficient permissions)
- `500`: Internal Server Error

## Security Considerations

- Passwords are hashed using bcrypt
- JWT tokens are signed with HS256 algorithm
- Player ID validation prevents unauthorized queue operations
- CORS is configured for Bevy game client connections
- WebSocket connections require valid JWT authentication

## Testing

The backend includes comprehensive tests covering:
- Authentication endpoints
- Matchmaking API functionality
- WebSocket connections using standard WebSocket library
- Database model operations
- Game server service functionality
- Error handling scenarios

Tests are compatible with Rust WebSocket clients by using standard WebSocket connections rather than transmit-client library.

## Development Setup

1. Install dependencies: `npm install`
2. Run migrations: `node ace migration:run`
3. Start development server: `npm run dev`
4. Run tests: `npm test`
5. Run linting: `npm run lint`