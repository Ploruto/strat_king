import { DateTime } from 'luxon'
import { BaseModel, column, belongsTo } from '@adonisjs/lucid/orm'
import type { BelongsTo } from '@adonisjs/lucid/types/relations'
import Player from './player.js'

export default class MatchmakingQueue extends BaseModel {
  @column({ isPrimary: true })
  declare id: number

  @column()
  declare playerId: number

  @column()
  declare gameMode: string

  @belongsTo(() => Player)
  declare player: BelongsTo<typeof Player>

  @column.dateTime({ autoCreate: true })
  declare createdAt: DateTime

  @column.dateTime({ autoCreate: true, autoUpdate: true })
  declare updatedAt: DateTime
}