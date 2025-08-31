import { spawn } from 'node:child_process'
import { promisify } from 'node:util'
import { exec } from 'node:child_process'

const execAsync = promisify(exec)

export class ServerManager {
  /**
   * Spawns a new game server container for a match
   */
  static async spawnGameServer(matchId: number, playerIds: number[], serverSecret: string): Promise<{
    containerId: string
    port: number
  }> {
    try {
      // Spawn Docker container with environment variables
      const dockerCommand = [
        'run', '-d', '-P', // -d for detached, -P for auto port mapping
        '-e', `SERVER_SECRET=${serverSecret}`,
        '-e', `MATCH_ID=${matchId}`,
        '-e', `EXPECTED_PLAYERS=${JSON.stringify(playerIds)}`,
        '-e', `BACKEND_URL=http://host.docker.internal:3333`,
        '-e', `SERVER_PORT=7777`,
        'strat-king-server:latest'
      ]

      console.log(`🐳 Spawning container for match ${matchId}...`)
      console.log(`Players: [${playerIds.join(', ')}]`)
      
      const { stdout: containerId } = await execAsync(`docker ${dockerCommand.join(' ')}`)
      const cleanContainerId = containerId.trim()

      // Get the assigned port
      const { stdout: portOutput } = await execAsync(
        `docker port ${cleanContainerId} 7777`
      )
      
      // Extract port number from output like "0.0.0.0:32768"
      const portMatch = portOutput.match(/:(\d+)/)
      const assignedPort = portMatch ? parseInt(portMatch[1]) : 7777

      console.log(`✅ Container spawned: ${cleanContainerId.substring(0, 12)}`)
      console.log(`🔌 Port mapped: 7777 -> ${assignedPort}`)

      return {
        containerId: cleanContainerId,
        port: assignedPort
      }
    } catch (error) {
      console.error('❌ Failed to spawn game server:', error)
      throw new Error(`Failed to spawn game server: ${error}`)
    }
  }

  /**
   * Stops and removes a game server container
   */
  static async stopGameServer(containerId: string): Promise<void> {
    try {
      console.log(`🛑 Stopping container: ${containerId.substring(0, 12)}`)
      
      // Stop the container gracefully
      await execAsync(`docker stop ${containerId}`)
      
      // Remove the container
      await execAsync(`docker rm ${containerId}`)
      
      console.log(`✅ Container cleaned up: ${containerId.substring(0, 12)}`)
    } catch (error) {
      console.error('⚠️ Failed to stop game server:', error)
      // Don't throw here - cleanup failures shouldn't break the flow
    }
  }

  /**
   * Lists all running game server containers
   */
  static async listGameServers(): Promise<Array<{ containerId: string, matchId: string }>> {
    try {
      const { stdout } = await execAsync(
        'docker ps --filter ancestor=strat-king-server:latest --format "{{.ID}},{{.Names}}"'
      )

      if (!stdout.trim()) {
        return []
      }

      return stdout.trim().split('\n').map(line => {
        const [containerId, name] = line.split(',')
        return {
          containerId,
          matchId: name || 'unknown'
        }
      })
    } catch (error) {
      console.error('Failed to list game servers:', error)
      return []
    }
  }
}