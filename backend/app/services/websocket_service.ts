import { WebSocketServer, WebSocket } from 'ws'
import { IncomingMessage } from 'http'
import { Secret } from '@adonisjs/core/helpers'
import Player from '#models/player'
import MatchmakingQueue from '#models/matchmaking_queue'
import Match from '#models/match'
import { ServerManager } from '#services/server_manager'

interface AuthenticatedWebSocket extends WebSocket {
  playerId?: number
  username?: string
}

export default class WebSocketService {
  private wss: WebSocketServer | null = null
  private static instance: WebSocketService
  private clients: Map<number, AuthenticatedWebSocket> = new Map()

  public static getInstance(): WebSocketService {
    if (!WebSocketService.instance) {
      WebSocketService.instance = new WebSocketService()
    }
    return WebSocketService.instance
  }

  public start(server?: any, port?: number) {
    if (server) {
      this.wss = new WebSocketServer({
        server,
        path: '/ws'
      })
    } else {
      this.wss = new WebSocketServer({ 
        port: port || 3334, 
        path: '/ws' 
      })
    }

    this.wss.on('connection', async (ws: AuthenticatedWebSocket, request: IncomingMessage) => {
      const baseUrl = server 
        ? `http://${request.headers.host}` 
        : `http://localhost:${port || 3334}`
      const url = new URL(request.url!, baseUrl)
      const token = url.searchParams.get('token')

      console.log("on connection: ")

      if (!token) {
        console.log("token missing")
        ws.close(1008, 'Token required')
        return
      }

      try {
        const token_secret = new Secret<string>(token)
        const res = await Player.accessTokens.verify(token_secret)
        
        if (!res) {
          console.log("invalid token; cancel")
          ws.close(1008, 'Invalid token')
          return
        }

        const player = await Player.find(res.tokenableId)
        if (!player) {
          ws.close(1008, 'Invalid token')
          return
        }

        ws.playerId = player.id
        ws.username = player.username
        this.clients.set(player.id, ws)

        console.log(`Player connected: ${player.username} (${player.id})`)

        ws.send(JSON.stringify({
          type: 'connection_success',
          data: { message: 'Connected to Strategy King server' }
        }))

        ws.on('message', (data) => {
          this.handleMessage(ws, data)
        })

        ws.on('close', async () => {
          if (ws.playerId) {
            this.clients.delete(ws.playerId)
            
            // Remove player from matchmaking queue on disconnect
            try {
              const deletedCount = await MatchmakingQueue.query()
                .where('playerId', ws.playerId)
                .delete()
              
              if (deletedCount > 0) {
                console.log(`Player ${ws.username} (${ws.playerId}) disconnected and removed from queue`)
              } else {
                console.log(`Player disconnected: ${ws.username} (${ws.playerId})`)
              }
            } catch (error) {
              console.error(`Error removing player ${ws.playerId} from queue on disconnect:`, error)
              console.log(`Player disconnected: ${ws.username} (${ws.playerId})`)
            }
          }
        })

      } catch (error) {
        ws.close(1008, 'Invalid token')
      }
    })
  }

  private async handleMessage(ws: AuthenticatedWebSocket, data: any) {
    try {
      const message = JSON.parse(data.toString())
      console.log(`Message from ${ws.username}:`, message)

      switch (message.type) {
        case 'queue_join':
          await this.handleQueueJoin(ws, message.data)
          break
        case 'queue_leave':
          await this.handleQueueLeave(ws, message.data)
          break
        case 'ping':
          ws.send(JSON.stringify({ type: 'pong' }))
          break
        default:
          ws.send(JSON.stringify({
            type: 'error',
            data: { message: `Unknown message type: ${message.type}` }
          }))
      }
    } catch (error) {
      ws.send(JSON.stringify({
        type: 'error',
        data: { message: 'Invalid message format' }
      }))
    }
  }

  private async handleQueueJoin(ws: AuthenticatedWebSocket, data: any) {
    if (!ws.playerId) {
      ws.send(JSON.stringify({
        type: 'queue_join_response',
        data: { success: false, message: 'Not authenticated' }
      }))
      return
    }

    try {
      // Check if player is already in queue
      const existingEntry = await MatchmakingQueue.query()
        .where('playerId', ws.playerId)
        .where('gameMode', '1v1')
        .first()

      if (existingEntry) {
        ws.send(JSON.stringify({
          type: 'queue_join_response',
          data: { success: false, message: 'Already in queue' }
        }))
        return
      }

      await MatchmakingQueue.create({
        playerId: ws.playerId,
        gameMode: '1v1'
      })

      ws.send(JSON.stringify({
        type: 'queue_join_response',
        data: { success: true, message: 'Joined queue successfully' }
      }))

      // Check if we have enough players for a match
      await this.tryCreateMatch()

    } catch (error) {
      console.error('Error handling queue join:', error)
      ws.send(JSON.stringify({
        type: 'queue_join_response',
        data: { success: false, message: 'Server error' }
      }))
    }
  }

  private async handleQueueLeave(ws: AuthenticatedWebSocket, data: any) {
    if (!ws.playerId) {
      ws.send(JSON.stringify({
        type: 'queue_leave_response',
        data: { success: false, message: 'Not authenticated' }
      }))
      return
    }

    try {
      const deletedCount = await MatchmakingQueue.query()
        .where('playerId', ws.playerId)
        .where('gameMode', '1v1')
        .delete()

      if (deletedCount > 0) {
        ws.send(JSON.stringify({
          type: 'queue_leave_response',
          data: { success: true, message: 'Left queue successfully' }
        }))
      } else {
        ws.send(JSON.stringify({
          type: 'queue_leave_response',
          data: { success: false, message: 'Not in queue' }
        }))
      }
    } catch (error) {
      console.error('Error handling queue leave:', error)
      ws.send(JSON.stringify({
        type: 'queue_leave_response',
        data: { success: false, message: 'Server error' }
      }))
    }
  }

  private async tryCreateMatch() {
    const queueEntries = await MatchmakingQueue.query()
      .where('gameMode', '1v1')
      .orderBy('createdAt', 'asc')
      .limit(2)

    if (queueEntries.length >= 2) {
      const playerIds = queueEntries.map(entry => entry.playerId)
      
      // Ensure we don't match a player against themselves
      const uniquePlayerIds = [...new Set(playerIds)]
      if (uniquePlayerIds.length < 2) {
        console.log('Cannot match - not enough unique players')
        return
      }

      const match = await Match.create({
        playerIds: playerIds,
        status: 'pending'
      })

      try {
        // Spawn game server container
        const { containerId, port } = await ServerManager.spawnGameServer(
          match.id,
          playerIds,
          match.serverSecret
        )

        // Update match with server details
        await match.merge({
          serverPort: port,
          authToken: containerId,
          status: 'spawning'
        }).save()

        console.log(`🎮 Match ${match.id} server spawning on port ${port}`)

        // Send match found notifications via WebSocket
        for (const playerId of playerIds) {
          const client = this.clients.get(playerId)
          if (client && client.readyState === 1) {
            client.send(JSON.stringify({
              type: 'match_found',
              data: {
                matchId: match.id,
                players: playerIds,
                status: 'spawning',
                serverHost: '127.0.0.1',
                serverPort: port,
                serverSecret: match.serverSecret,
                message: 'Game server is starting...'
              }
            }))
          }
        }

        // Remove these players from the queue
        await MatchmakingQueue.query()
          .whereIn('id', queueEntries.map(entry => entry.id))
          .delete()

      } catch (error) {
        console.error('Failed to spawn game server:', error)

        // Update match as failed
        await match.merge({ status: 'failed' }).save()

        // Notify players of failure
        for (const playerId of playerIds) {
          const client = this.clients.get(playerId)
          if (client && client.readyState === 1) {
            client.send(JSON.stringify({
              type: 'match_failed',
              data: {
                matchId: match.id,
                error: 'Failed to start game server'
              }
            }))
          }
        }
      }
    }
  }

  public sendToPlayer(playerId: number, message: any) {
    const client = this.clients.get(playerId)
    if (client && client.readyState === WebSocket.OPEN) {
      client.send(JSON.stringify(message))
    }
  }

  public broadcast(message: any) {
    this.clients.forEach((client) => {
      if (client.readyState === WebSocket.OPEN) {
        client.send(JSON.stringify(message))
      }
    })
  }

  public getConnectedPlayers(): number[] {
    return Array.from(this.clients.keys())
  }

  public getClients(): Map<number, AuthenticatedWebSocket> {
    return this.clients
  }
}
