import { WebSocketServer } from 'ws'
import jwt from 'jsonwebtoken'
import env from '#start/env'
import Player from '#models/player'

let wss: WebSocketServer | null = null
const clients = new Map()

export function startWebSocketServer(server: any) {
  wss = new WebSocketServer({ server, path: '/ws' })

  wss.on('connection', async (ws, request) => {
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

      clients.set(player.id, ws)
      console.log(`Player connected: ${player.username} (${player.id})`)

      ws.send(JSON.stringify({
        type: 'connection_success',
        data: { message: 'Connected to Strategy King server' }
      }))

      ws.on('message', (data) => {
        try {
          const message = JSON.parse(data.toString())
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
      })

      ws.on('close', () => {
        clients.delete(player.id)
        console.log(`Player disconnected: ${player.username} (${player.id})`)
      })

    } catch (error) {
      ws.close(1008, 'Invalid token')
    }
  })
}

export function getWebSocketServer() {
  return wss
}

export function getConnectedClients() {
  return clients
}