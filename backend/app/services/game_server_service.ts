import { spawn } from 'node:child_process'
import jwt from 'jsonwebtoken'
import env from '#start/env'

export default class GameServerService {
  private usedPorts = new Set<number>()
  private readonly startPort = 5001
  private readonly maxPorts = 100

  async allocatePort(): Promise<number> {
    for (let i = 0; i < this.maxPorts; i++) {
      const port = this.startPort + i
      if (!this.usedPorts.has(port)) {
        this.usedPorts.add(port)
        return port
      }
    }
    throw new Error('No available ports for game server')
  }

  releasePort(port: number) {
    this.usedPorts.delete(port)
  }

  async generateGameToken(playerIds: number[]): Promise<string> {
    const payload = {
      match_id: `match_${Date.now()}`,
      player_ids: playerIds,
      exp: Math.floor(Date.now() / 1000) + (15 * 60) // 15 minutes
    }

    return jwt.sign(payload, env.get('APP_KEY'), { algorithm: 'HS256' })
  }

  async startGameServer(matchId: number, port: number, authToken: string, playerIds: number[]) {
    const serverPath = '../server'
    
    const gameServerProcess = spawn('cargo', ['run'], {
      cwd: serverPath,
      env: {
        ...process.env,
        AUTH_TOKEN_SECRET: env.get('APP_KEY'),
        MATCH_ID: matchId.toString(),
        EXPECTED_PLAYERS: playerIds.length.toString(),
        SERVER_PORT: port.toString()
      },
      stdio: 'pipe'
    })

    gameServerProcess.stdout?.on('data', (data) => {
      console.log(`[Game Server ${port}]: ${data}`)
    })

    gameServerProcess.stderr?.on('data', (data) => {
      console.error(`[Game Server ${port} Error]: ${data}`)
    })

    gameServerProcess.on('close', (code) => {
      console.log(`Game server on port ${port} exited with code ${code}`)
      this.releasePort(port)
    })

    gameServerProcess.on('error', (error) => {
      console.error(`Failed to start game server on port ${port}:`, error)
      this.releasePort(port)
    })

    return gameServerProcess
  }
}