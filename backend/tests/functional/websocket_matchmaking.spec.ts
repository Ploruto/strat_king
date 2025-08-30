import { test } from '@japa/runner'
import testUtils from '@adonisjs/core/services/test_utils'
import WebSocket from 'ws'
import { createWebSocketServer } from '#start/ws_init'
import Player from '#models/player'
import Match from '#models/match'
import hash from '@adonisjs/core/services/hash'
import jwt from 'jsonwebtoken'
import env from '#start/env'

test.group('WebSocket Matchmaking', (group) => {
  group.each.setup(() => testUtils.db().withGlobalTransaction())

  let wsServer: any

  group.setup(async () => {
    wsServer = createWebSocketServer(8081)
  })

  group.teardown(async () => {
    wsServer?.close()
  })

  test('should send match_found notifications via WebSocket when match is created', async ({ client, assert }) => {
    // Create two players
    const player1 = await Player.create({
      username: 'ws_player1',
      passwordHash: await hash.make('password123')
    })

    const player2 = await Player.create({
      username: 'ws_player2',
      passwordHash: await hash.make('password123')
    })

    // Create tokens for API calls
    const token1 = await Player.accessTokens.create(player1)
    const token2 = await Player.accessTokens.create(player2)

    // Create JWT tokens for WebSocket connections
    const wsToken1 = jwt.sign({ playerId: player1.id }, env.get('APP_KEY'))
    const wsToken2 = jwt.sign({ playerId: player2.id }, env.get('APP_KEY'))

    // Connect both players via WebSocket
    const ws1 = new WebSocket(`ws://localhost:8081/ws?token=${wsToken1}`)
    const ws2 = new WebSocket(`ws://localhost:8081/ws?token=${wsToken2}`)

    // Wait for connections to be established
    await Promise.all([
      new Promise((resolve) => ws1.once('open', resolve)),
      new Promise((resolve) => ws2.once('open', resolve))
    ])

    // Clear connection messages
    await Promise.all([
      new Promise((resolve) => ws1.once('message', resolve)),
      new Promise((resolve) => ws2.once('message', resolve))
    ])

    // Set up message listeners to catch match_found notifications
    const matchFoundMessages: any[] = []

    ws1.on('message', (data) => {
      const message = JSON.parse(data.toString())
      console.log(data.toString())
      if (message.type === 'match_found') {
        matchFoundMessages.push({ player: player1.id, message })
      }
    })

    ws2.on('message', (data) => {
      console.log(data.toString())
      const message = JSON.parse(data.toString())
      if (message.type === 'match_found') {
        matchFoundMessages.push({ player: player2.id, message })
      }
    })

    // Player 1 joins queue
    await client
      .post('/matchmaking/join')
      .header('Authorization', `Bearer ${token1.value!.release()}`)

    // Player 2 joins queue - this should trigger match creation and notifications
    await client
      .post('/matchmaking/join')
      .header('Authorization', `Bearer ${token2.value!.release()}`)

    // Wait briefly for WebSocket messages to be processed
    await new Promise(resolve => setTimeout(resolve, 100))

    // Verify match was created
    const matches = await Match.all()
    assert.lengthOf(matches, 1)

    // Verify both players received match_found notifications
    assert.lengthOf(matchFoundMessages, 2)

    // Check first notification
    const msg1 = matchFoundMessages.find(m => m.player === player1.id)
    assert.isNotNull(msg1)
    assert.equal(msg1.message.type, 'match_found')
    assert.equal(msg1.message.data.matchId, matches[0].id)
    assert.deepEqual(msg1.message.data.players.sort(), [player1.id, player2.id].sort())
    assert.equal(msg1.message.data.status, 'pending')

    // Check second notification
    const msg2 = matchFoundMessages.find(m => m.player === player2.id)
    assert.isNotNull(msg2)
    assert.equal(msg2.message.type, 'match_found')
    assert.equal(msg2.message.data.matchId, matches[0].id)
    assert.deepEqual(msg2.message.data.players.sort(), [player1.id, player2.id].sort())
    assert.equal(msg2.message.data.status, 'pending')

    // Clean up connections
    ws1.close()
    ws2.close()
  })
})
