import type { HttpContext } from '@adonisjs/core/http'
import hash from '@adonisjs/core/services/hash'
import Player from '#models/player'
import { loginValidator, registerValidator } from '#validators/auth'

export default class AuthController {
  async login({ auth, request, response }: HttpContext) {
    try {
      const { username, password } = await request.validateUsing(loginValidator)

      const player = await Player.findByOrFail('username', username)
      
      const isPasswordValid = await hash.verify(player.passwordHash, password)
      if (!isPasswordValid) {
        return response.status(401).json({
          success: false,
          token: null,
          player_id: null,
          message: 'Invalid credentials'
        })
      }

      const token = await auth.use('api').generate(player)

      return response.json({
        success: true,
        token: token.value!.release(),
        player_id: player.id,
        message: 'Authentication successful'
      })
    } catch (error) {
      return response.status(401).json({
        success: false,
        token: null,
        player_id: null,
        message: 'Invalid credentials'
      })
    }
  }

  async register({ auth, request, response }: HttpContext) {
    try {
      const { username, password } = await request.validateUsing(registerValidator)

      const existingPlayer = await Player.findBy('username', username)
      if (existingPlayer) {
        return response.status(400).json({
          success: false,
          token: null,
          player_id: null,
          message: 'Username already exists'
        })
      }

      const hashedPassword = await hash.make(password)
      const player = await Player.create({
        username,
        passwordHash: hashedPassword
      })

      const token = await auth.use('api').generate(player)

      return response.json({
        success: true,
        token: token.value!.release(),
        player_id: player.id,
        message: 'Registration successful'
      })
    } catch (error) {
      return response.status(400).json({
        success: false,
        token: null,
        player_id: null,
        message: 'Registration failed'
      })
    }
  }
}