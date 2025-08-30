import type { HttpContext } from '@adonisjs/core/http'
import MatchmakingQueue from '#models/matchmaking_queue'
import { matchmakingValidator } from '#validators/matchmaking'

export default class MatchmakingController {
  async join({ auth, request, response }: HttpContext) {
    try {
      await auth.authenticate()
      const player = auth.user!

      const { game_mode, player_id } = await request.validateUsing(matchmakingValidator)

      if (player.id !== player_id) {
        return response.status(403).json({
          success: false,
          message: 'Unauthorized'
        })
      }

      const existingQueueEntry = await MatchmakingQueue.findBy('player_id', player_id)
      if (existingQueueEntry) {
        return response.status(400).json({
          success: false,
          message: 'Player already in queue'
        })
      }

      await MatchmakingQueue.create({
        playerId: player_id,
        gameMode: game_mode
      })

      const queueCount = await MatchmakingQueue.query()
        .where('game_mode', game_mode)
        .count('* as total')
      
      const queuePosition = queueCount[0].$extras.total

      return response.json({
        success: true,
        queue_position: queuePosition,
        estimated_wait_seconds: queuePosition * 15
      })
    } catch (error) {
      return response.status(400).json({
        success: false,
        message: 'Failed to join matchmaking queue'
      })
    }
  }

  async leave({ auth, request, response }: HttpContext) {
    try {
      await auth.authenticate()
      const player = auth.user!

      const { player_id } = request.only(['player_id'])

      if (player.id !== player_id) {
        return response.status(403).json({
          success: false,
          message: 'Unauthorized'
        })
      }

      const queueEntry = await MatchmakingQueue.findBy('player_id', player_id)
      if (queueEntry) {
        await queueEntry.delete()
      }

      return response.json({
        success: true,
        message: 'Left matchmaking queue'
      })
    } catch (error) {
      return response.status(400).json({
        success: false,
        message: 'Failed to leave matchmaking queue'
      })
    }
  }
}