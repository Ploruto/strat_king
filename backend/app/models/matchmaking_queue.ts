import { DateTime } from 'luxon'
import { BaseModel, column } from '@adonisjs/lucid/orm'

export default class MatchmakingQueue extends BaseModel {
  @column({ isPrimary: true })
  declare id: number

  @column()
  declare playerId: number

  @column()
  declare gameMode: string

  @column.dateTime({ autoCreate: true })
  declare createdAt: DateTime

  @column.dateTime({ autoCreate: true, autoUpdate: true })
  declare updatedAt: DateTime
}