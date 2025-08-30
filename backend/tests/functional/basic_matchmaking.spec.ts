import { test } from '@japa/runner'
import testUtils from '@adonisjs/core/services/test_utils'
import MatchmakingQueue from '#models/matchmaking_queue'
import Player from '#models/player'
import Match from '#models/match'
import hash from '@adonisjs/core/services/hash'

test.group('Basic Matchmaking', (group) => {
  group.each.setup(() => testUtils.db().withGlobalTransaction())

  test('should respond to join queue endpoint with auth', async ({ client }) => {
    const player = await Player.create({
      username: 'authplayer',
      passwordHash: await hash.make('password123')
    })

    const token = await Player.accessTokens.create(player)

    const response = await client
      .post('/matchmaking/join')
      .header('Authorization', `Bearer ${token.value!.release()}`)

    response.assertStatus(200)
    response.assertBodyContains({ success: true })
  })

  test('should create queue entry in database', async ({ client, assert }) => {
    const player = await Player.create({
      username: 'testplayer',
      passwordHash: await hash.make('password123')
    })

    const token = await Player.accessTokens.create(player)

    const response = await client
      .post('/matchmaking/join')
      .header('Authorization', `Bearer ${token.value!.release()}`)

    response.assertStatus(200)

    const queueEntries = await MatchmakingQueue.all()
    assert.lengthOf(queueEntries, 1)
    assert.equal(queueEntries[0].playerId, player.id)
  })

  test('should require authentication to join queue', async ({ client }) => {
    const response = await client.post('/matchmaking/join')

    response.assertStatus(401)
  })

  test('should allow same player to join queue multiple times', async ({ client, assert }) => {
    const player = await Player.create({
      username: 'duplicatetest',
      passwordHash: await hash.make('password123')
    })

    const token = await Player.accessTokens.create(player)

    // First join
    const response1 = await client
      .post('/matchmaking/join')
      .header('Authorization', `Bearer ${token.value!.release()}`)

    response1.assertStatus(200)

    // Verify first entry is in queue
    let queueEntries = await MatchmakingQueue.all()
    assert.lengthOf(queueEntries, 1)
    assert.equal(queueEntries[0].playerId, player.id)

    // Second join - this should create a match since same player counts as 2
    const response2 = await client
      .post('/matchmaking/join')
      .header('Authorization', `Bearer ${token.value!.release()}`)

    response2.assertStatus(200)

    // After 2 joins from same player, should create match and clear queue
    queueEntries = await MatchmakingQueue.all()
    assert.lengthOf(queueEntries, 0)
    
    // Verify match was created
    const matches = await Match.all()
    assert.lengthOf(matches, 1)
    assert.include(matches[0].playerIds, player.id)
  })

  test('should create match when two different players join queue', async ({ client, assert }) => {
    const player1 = await Player.create({
      username: 'multiplayer1',
      passwordHash: await hash.make('password123')
    })

    const player2 = await Player.create({
      username: 'multiplayer2', 
      passwordHash: await hash.make('password123')
    })

    const token1 = await Player.accessTokens.create(player1)
    const token2 = await Player.accessTokens.create(player2)

    // First player joins
    const response1 = await client
      .post('/matchmaking/join')
      .header('Authorization', `Bearer ${token1.value!.release()}`)

    response1.assertStatus(200)

    // Verify first player is in queue
    let queueEntries = await MatchmakingQueue.all()
    assert.lengthOf(queueEntries, 1)
    assert.equal(queueEntries[0].playerId, player1.id)

    // Second player joins - should trigger match creation
    const response2 = await client
      .post('/matchmaking/join')
      .header('Authorization', `Bearer ${token2.value!.release()}`)

    response2.assertStatus(200)

    // After match creation, queue should be empty
    queueEntries = await MatchmakingQueue.all()
    assert.lengthOf(queueEntries, 0)
    
    // Verify match was created with both players
    const matches = await Match.all()
    assert.lengthOf(matches, 1)
    
    const match = matches[0]
    assert.lengthOf(match.playerIds, 2)
    assert.include(match.playerIds, player1.id)
    assert.include(match.playerIds, player2.id)
  })

  test('should create queue entry with correct data integrity', async ({ client, assert }) => {
    const player = await Player.create({
      username: 'integritytest',
      passwordHash: await hash.make('password123')
    })

    const token = await Player.accessTokens.create(player)

    const response = await client
      .post('/matchmaking/join')
      .header('Authorization', `Bearer ${token.value!.release()}`)

    response.assertStatus(200)

    const queueEntry = await MatchmakingQueue.firstOrFail()
    
    // Verify correct player ID
    assert.equal(queueEntry.playerId, player.id)
    
    // Verify game mode is set correctly
    assert.equal(queueEntry.gameMode, '1v1')
  })

  test('should reject request with invalid token', async ({ client }) => {
    const response = await client
      .post('/matchmaking/join')
      .header('Authorization', 'Bearer invalid_token_here')

    response.assertStatus(401)
  })
})
