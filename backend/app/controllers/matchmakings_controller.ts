import type { HttpContext } from '@adonisjs/core/http'
import MatchmakingQueue from '#models/matchmaking_queue'

export default class MatchmakingsController {
  async join({ auth }: HttpContext) {
    const player = auth.getUserOrFail()
    
    await MatchmakingQueue.create({
      playerId: player.id,
      gameMode: '1v1'
    })

    return { success: true }
  }
}