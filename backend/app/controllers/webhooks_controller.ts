import type { HttpContext } from '@adonisjs/core/http'
import Match from '#models/match'
import { clients } from '#start/ws_init'

export default class WebhooksController {
  async serverReady({ request, response }: HttpContext) {
    const { match_id } = request.body()

    if (!match_id) {
      return response.badRequest({ error: 'match_id is required' })
    }

    try {
      const match = await Match.findOrFail(match_id)
      
      // Update match status to active
      await match.merge({ status: 'active' }).save()

      // Notify players via WebSocket that server is ready
      for (const playerId of match.playerIds) {
        const client = clients.get(playerId)
        if (client && client.readyState === 1) { // 1 = OPEN
          client.send(JSON.stringify({
            type: 'server_ready',
            data: {
              matchId: match.id,
              serverAddress: 'localhost',
              serverPort: match.serverPort || 7777,
              serverSecret: match.serverSecret
            }
          }))
        }
      }

      console.log(`✅ Match ${match_id} server is ready, players notified`)
      return { success: true, message: 'Server ready notification processed' }
    } catch (error) {
      console.error('Error processing server ready webhook:', error)
      return response.internalServerError({ error: 'Failed to process server ready notification' })
    }
  }

  async matchComplete({ request, response }: HttpContext) {
    const { match_id, winner } = request.body()

    if (!match_id) {
      return response.badRequest({ error: 'match_id is required' })
    }

    try {
      const match = await Match.findOrFail(match_id)
      
      // Update match status to completed
      await match.merge({ status: 'completed' }).save()

      // Notify players via WebSocket that match is complete
      for (const playerId of match.playerIds) {
        const client = clients.get(playerId)
        if (client && client.readyState === 1) {
          client.send(JSON.stringify({
            type: 'match_complete',
            data: {
              matchId: match.id,
              winner: winner || null,
              status: 'completed'
            }
          }))
        }
      }

      console.log(`🏁 Match ${match_id} completed${winner ? ` - Winner: ${winner}` : ''}`)
      return { success: true, message: 'Match completion processed' }
    } catch (error) {
      console.error('Error processing match complete webhook:', error)
      return response.internalServerError({ error: 'Failed to process match completion' })
    }
  }
}