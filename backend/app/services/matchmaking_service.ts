import transmit from '@adonisjs/transmit/services/main'
import MatchmakingQueue from '#models/matchmaking_queue'
import Match from '#models/match'
import GameServerService from './game_server_service.js'

export default class MatchmakingService {
  private static instance: MatchmakingService
  private gameServerService = new GameServerService()

  public static getInstance(): MatchmakingService {
    if (!MatchmakingService.instance) {
      MatchmakingService.instance = new MatchmakingService()
    }
    return MatchmakingService.instance
  }

  async processQueue() {
    const queueEntries = await MatchmakingQueue.query()
      .where('game_mode', '1v1')
      .orderBy('created_at', 'asc')
      .limit(2)
      .preload('player')

    if (queueEntries.length >= 2) {
      await this.createMatch(queueEntries)
    }

    this.broadcastQueueStatus()
  }

  private async createMatch(queueEntries: MatchmakingQueue[]) {
    const playerIds = queueEntries.map(entry => entry.playerId)
    
    const serverPort = await this.gameServerService.allocatePort()
    const authToken = await this.gameServerService.generateGameToken(playerIds)
    
    const match = await Match.create({
      status: 'active',
      playerIds,
      serverPort,
      authToken
    })

    await this.gameServerService.startGameServer(match.id, serverPort, authToken, playerIds)

    for (let i = 0; i < queueEntries.length; i++) {
      const entry = queueEntries[i]
      const playerId = entry.playerId

      transmit.broadcast(`player_${playerId}`, {
        type: 'match_found',
        match_id: match.id,
        server_host: 'localhost',
        server_port: serverPort,
        auth_token: authToken,
        team: i
      })

      await entry.delete()
    }
  }

  async broadcastQueueStatus() {
    const queueEntries = await MatchmakingQueue.query()
      .where('game_mode', '1v1')
      .orderBy('created_at', 'asc')
      .preload('player')

    for (let i = 0; i < queueEntries.length; i++) {
      const entry = queueEntries[i]
      const position = i + 1
      const estimatedWait = position * 15

      transmit.broadcast(`player_${entry.playerId}`, {
        type: 'queue_status',
        position,
        estimated_wait: estimatedWait
      })
    }
  }

  async removePlayerFromQueue(playerId: number) {
    const queueEntry = await MatchmakingQueue.findBy('player_id', playerId)
    if (queueEntry) {
      await queueEntry.delete()
      
      transmit.broadcast(`player_${playerId}`, {
        type: 'queue_cancelled',
        reason: 'Player disconnected'
      })
    }
  }
}