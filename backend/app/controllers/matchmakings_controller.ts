import type { HttpContext } from '@adonisjs/core/http'
import MatchmakingQueue from '#models/matchmaking_queue'
import Match from '#models/match'
import { clients } from '#start/ws_init'
import { ServerManager } from '#services/server_manager'

export default class MatchmakingsController {
  async join({ auth }: HttpContext) {
    await auth.authenticate()
    const player = auth.getUserOrFail()

    // Check if player is already in queue
    const existingEntry = await MatchmakingQueue.query()
      .where('playerId', player.id)
      .where('gameMode', '1v1')
      .first()

    if (existingEntry) {
      return { success: false, message: 'Already in queue' }
    }

    await MatchmakingQueue.create({
      playerId: player.id,
      gameMode: '1v1'
    })

    // Check if we have enough players for a match
    const queueEntries = await MatchmakingQueue.query()
      .where('gameMode', '1v1')
      .orderBy('createdAt', 'asc')
      .limit(2)

    if (queueEntries.length >= 2) {
      // Create match with the first 2 players
      const playerIds = queueEntries.map(entry => entry.playerId)
      
      // Ensure we don't match a player against themselves
      const uniquePlayerIds = [...new Set(playerIds)]
      if (uniquePlayerIds.length < 2) {
        console.log('Cannot match - not enough unique players')
        return { success: true }
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
          authToken: containerId, // Store container ID in authToken for cleanup
          status: 'spawning'
        }).save()

        console.log(`🎮 Match ${match.id} server spawning on port ${port}`)

        // Send match found notifications via WebSocket
        for (const playerId of playerIds) {
          const client = clients.get(playerId)
          if (client && client.readyState === 1) { // 1 = OPEN
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
          const client = clients.get(playerId)
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

    return { success: true }
  }
}
