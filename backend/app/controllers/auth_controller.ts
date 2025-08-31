import type { HttpContext } from '@adonisjs/core/http'
import hash from '@adonisjs/core/services/hash'
import Player from '#models/player'
import jwt from 'jsonwebtoken'
import env from '#start/env'

export default class AuthController {
  async register({ request, response }: HttpContext) {
    const { username, password } = request.only(['username', 'password'])

    const existingPlayer = await Player.findBy('username', username)
    if (existingPlayer) {
      return response.status(400).json({
        success: false,
        message: 'Username already exists'
      })
    }

    const passwordHash = await hash.make(password)
    const player = await Player.create({
      username,
      passwordHash
    })

    const token = jwt.sign({ playerId: player.id }, env.get('APP_KEY'), { expiresIn: '24h' })

    return response.status(201).json({
      success: true,
      message: 'Player registered successfully',
      data: {
        player_id: player.id,
        username: player.username,
        token
      }
    })
  }

  async login({ request, response }: HttpContext) {
    console.log("login")
    const { username, password } = request.only(['username', 'password'])
    console.log(`received request with: ${username} ${password}`)

    const player = await Player.findBy('username', username)
    if (!player) {
      return response.status(401).json({
        success: false,
        message: 'Invalid credentials'
      })
    }

    const isValidPassword = await hash.verify(player.passwordHash, password)
    if (!isValidPassword) {
      return response.status(401).json({
        success: false,
        message: 'Invalid credentials'
      })
    }

    const token = jwt.sign({ playerId: player.id }, env.get('APP_KEY'), { expiresIn: '24h' })

    return response.status(200).json({
      success: true,
      message: 'Login successful',
      data: {
        player_id: player.id,
        username: player.username,
        token
      }
    })
  }
}
