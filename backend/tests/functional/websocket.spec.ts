import { test } from '@japa/runner'
import testUtils from '@adonisjs/core/services/test_utils'
import Player from '#models/player'
import hash from '@adonisjs/core/services/hash'
import jwt from 'jsonwebtoken'
import env from '#start/env'
import WebSocket from 'ws'
import { createWebSocketServer } from '#start/ws_init'

test.group('WebSocket', (group) => {
  let wss: any

  group.setup(async () => {
    wss = createWebSocketServer(8081)
  })

  group.teardown(async () => {
    if (wss) {
      wss.close()
    }
  })

  group.each.setup(() => testUtils.db().withGlobalTransaction())

  test('should establish WebSocket connection with valid token', async ({ assert }) => {
    // Setup: Create a test player and generate JWT token
    const player = await Player.create({
      username: 'wsuser',
      passwordHash: await hash.make('password123')
    })

    const token = jwt.sign({ playerId: player.id }, env.get('APP_KEY'), { expiresIn: '24h' })

    // Test flow: Connect → Wait for welcome message → Verify connection → Close
    return new Promise<void>((resolve, reject) => {
      // Step 1: Attempt WebSocket connection with valid JWT token
      const ws = new WebSocket(`ws://localhost:8081/ws?token=${token}`)

      // Safety timeout to prevent hanging tests
      const timeout = setTimeout(() => {
        ws.close()
        reject(new Error('Connection timeout'))
      }, 5000)

      // Step 2: Listen for server messages (should receive welcome message)
      ws.on('message', (data) => {
        const message = JSON.parse(data.toString())
        
        // Step 3: Verify we received the expected connection success message
        if (message.type === 'connection_success') {
          clearTimeout(timeout)
          assert.isTrue(true, 'WebSocket connection established')
          ws.close()
          resolve() // Test passes
        }
      })

      // Handle connection errors
      ws.on('error', (error) => {
        clearTimeout(timeout)
        reject(error)
      })
    })
  }).timeout(6000)

  test('should send and receive echo messages', async ({ assert }) => {
    // Setup: Create test player and JWT token for authentication
    const player = await Player.create({
      username: 'msguser',
      passwordHash: await hash.make('password123')
    })

    const token = jwt.sign({ playerId: player.id }, env.get('APP_KEY'), { expiresIn: '24h' })

    // Test flow: Connect → Receive welcome → Send test message → Receive echo → Verify → Close
    return new Promise<void>((resolve, reject) => {
      // Step 1: Establish WebSocket connection with valid token
      const ws = new WebSocket(`ws://localhost:8081/ws?token=${token}`)

      // Safety timeout to prevent hanging tests
      const timeout = setTimeout(() => {
        ws.close()
        reject(new Error('Test timeout'))
      }, 5000)

      let connectionEstablished = false

      // Step 2: Handle all messages from server (event-driven, non-linear flow)
      ws.on('message', (data) => {
        const message = JSON.parse(data.toString())
        console.log(message)

        // First message: Server sends welcome message after successful connection
        if (message.type === 'connection_success' && !connectionEstablished) {
          connectionEstablished = true
          
          // Step 3: Send our test message to the server
          const testMessage = { type: 'test', content: 'Hello WebSocket!' }
          ws.send(JSON.stringify(testMessage))
          
        // Second message: Server echoes back our test message
        } else if (message.type === 'echo') {
          clearTimeout(timeout)
          
          // Step 4: Verify the echo contains our original message
          assert.isObject(message.data.received, 'Should receive echoed message')
          assert.equal(message.data.received.content, 'Hello WebSocket!')
          
          ws.close()
          resolve() // Test passes
        }
      })

      // Handle connection errors
      ws.on('error', (error) => {
        clearTimeout(timeout)
        reject(error)
      })
    })
  }).timeout(6000)
})
