import { WebSocketServer, WebSocket } from 'ws'
import { IncomingMessage } from 'http'
import jwt from 'jsonwebtoken'
import env from '#start/env'
import Player from '#models/player'

interface AuthenticatedWebSocket extends WebSocket {
  playerId?: number
  username?: string
}

export default class WebSocketService {
  private wss: WebSocketServer | null = null
  private clients: Map<number, AuthenticatedWebSocket> = new Map()

  public start(server: any) {
    this.wss = new WebSocketServer({ 
      server,
      path: '/ws'
    })

    this.wss.on('connection', async (ws: AuthenticatedWebSocket, request: IncomingMessage) => {
      const url = new URL(request.url!, `http://${request.headers.host}`)
      const token = url.searchParams.get('token')

      if (!token) {
        ws.close(1008, 'Token required')
        return
      }

      try {
        const payload = jwt.verify(token, env.get('APP_KEY')) as { playerId: number }
        const player = await Player.find(payload.playerId)

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

        ws.on('close', () => {
          if (ws.playerId) {
            this.clients.delete(ws.playerId)
            console.log(`Player disconnected: ${ws.username} (${ws.playerId})`)
          }
        })

      } catch (error) {
        ws.close(1008, 'Invalid token')
      }
    })
  }

  private handleMessage(ws: AuthenticatedWebSocket, data: any) {
    try {
      const message = JSON.parse(data.toString())
      console.log(`Message from ${ws.username}:`, message)

      ws.send(JSON.stringify({
        type: 'echo',
        data: { received: message }
      }))
    } catch (error) {
      ws.send(JSON.stringify({
        type: 'error',
        data: { message: 'Invalid message format' }
      }))
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
}