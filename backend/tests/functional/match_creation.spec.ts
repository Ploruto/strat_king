import { test } from '@japa/runner'
import testUtils from '@adonisjs/core/services/test_utils'
import MatchmakingQueue from '#models/matchmaking_queue'
import Player from '#models/player'
import Match from '#models/match'
import hash from '@adonisjs/core/services/hash'

test.group('Match Creation', (group) => {
  group.each.setup(() => testUtils.db().withGlobalTransaction())

  test('should create match when 2 players join queue', async ({ client, assert }) => {
    const player1 = await Player.create({
      username: 'match_player1',
      passwordHash: await hash.make('password123')
    })

    const player2 = await Player.create({
      username: 'match_player2',
      passwordHash: await hash.make('password123')
    })

    const token1 = await Player.accessTokens.create(player1)
    const token2 = await Player.accessTokens.create(player2)

    // First player joins
    await client
      .post('/matchmaking/join')
      .header('Authorization', `Bearer ${token1.value!.release()}`)

    // Second player joins - this should trigger match creation
    await client
      .post('/matchmaking/join')
      .header('Authorization', `Bearer ${token2.value!.release()}`)

    // Verify match was created
    const matches = await Match.all()
    assert.lengthOf(matches, 1)
    
    const match = matches[0]
    assert.equal(match.status, 'pending')
    assert.lengthOf(match.playerIds, 2)
    assert.include(match.playerIds, player1.id)
    assert.include(match.playerIds, player2.id)

    // Verify queue entries were removed
    const queueEntries = await MatchmakingQueue.all()
    assert.lengthOf(queueEntries, 0)
  })
})