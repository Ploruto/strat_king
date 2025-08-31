import { WebSocketServer } from 'ws'
import Player from '#models/player'
import { Secret } from '@adonisjs/core/helpers'

const clients = new Map()

function createWebSocketServer(port: number = 8080) {
  const wss = new WebSocketServer({ port, path: '/ws' })

  wss.on('connection', async (ws, request) => {
    const url = new URL(request.url!, `http://localhost:${port}`)
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
      if(!res) {
        console.log("invalid token; cancel")
        ws.close(1008, 'Invalid token')
        return
      }

      clients.set(res.tokenableId, ws)

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
        clients.delete(res.tokenableId)
      })

    } catch (error) {
      ws.close(1008, 'Invalid token')
    }
  })

  return wss
}

export { createWebSocketServer, clients }
