import { test } from '@japa/runner'
import { ApiClient } from '@japa/api-client'
import testUtils from '@adonisjs/core/services/test_utils'
import Player from '#models/player'
import hash from '@adonisjs/core/services/hash'

test.group('Auth', (group) => {
  group.each.setup(() => testUtils.db().withGlobalTransaction())

  test('should register a new player', async ({ client, assert }) => {
    const response = await client.post('/auth/register').json({
      username: 'testuser',
      password: 'password123'
    })

    response.assertStatus(201)
    response.assertBodyContains({
      success: true,
      message: 'Player registered successfully'
    })

    const body = response.body()
    assert.properties(body.data, ['player_id', 'username', 'token'])
    assert.equal(body.data.username, 'testuser')

    const player = await Player.find(body.data.player_id)
    assert.isNotNull(player)
    assert.equal(player!.username, 'testuser')

  })

  test('should not register player with existing username', async ({ client }) => {
    await Player.create({
      username: 'existinguser',
      passwordHash: 'hashedpassword'
    })

    const response = await client.post('/auth/register').json({
      username: 'existinguser',
      password: 'password123'
    })

    response.assertStatus(400)
    response.assertBodyContains({
      success: false,
      message: 'Username already exists'
    })
  })

  test('should login with valid credentials', async ({ client, assert }) => {
    const player = await Player.create({
      username: 'loginuser',
      passwordHash: await hash.make('password123')
    })

    const response = await client.post('/auth/login').json({
      username: 'loginuser',
      password: 'password123'
    })

    response.assertStatus(200)
    response.assertBodyContains({
      success: true,
      message: 'Login successful'
    })

    const body = response.body()
    assert.properties(body.data, ['player_id', 'username', 'token'])
    assert.equal(body.data.player_id, player.id)
    assert.equal(body.data.username, 'loginuser')
  })

  test('should not login with invalid username', async ({ client }) => {
    const response = await client.post('/auth/login').json({
      username: 'nonexistentuser',
      password: 'password123'
    })

    response.assertStatus(401)
    response.assertBodyContains({
      success: false,
      message: 'Invalid credentials'
    })
  })

  test('should not login with invalid password', async ({ client }) => {
    await Player.create({
      username: 'testuser2',
      passwordHash: await hash.make('correctpassword')
    })

    const response = await client.post('/auth/login').json({
      username: 'testuser2',
      password: 'wrongpassword'
    })

    response.assertStatus(401)
    response.assertBodyContains({
      success: false,
      message: 'Invalid credentials'
    })
  })
})
