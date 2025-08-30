import type { HttpContext } from '@adonisjs/core/http'
import transmit from '@adonisjs/transmit/services/main'
import jwt from 'jsonwebtoken'
import env from '#start/env'
import Player from '#models/player'
import MatchmakingService from '#services/matchmaking_service'

export default class MatchmakingWebSocketController {
  async connect({ request, response }: HttpContext) {
    const token = request.input('token')
    
    if (!token) {
      return response.status(401).json({ error: 'Token required' })
    }

    try {
      const payload = jwt.verify(token, env.get('APP_KEY')) as any
      const player = await Player.find(payload.sub)
      
      if (!player) {
        return response.status(401).json({ error: 'Invalid token' })
      }

      const matchmakingService = MatchmakingService.getInstance()
      
      transmit.subscribe(`player_${player.id}`)
      
      setInterval(() => {
        matchmakingService.processQueue()
      }, 5000)

      return response.json({ success: true })
    } catch (error) {
      return response.status(401).json({ error: 'Invalid token' })
    }
  }
}