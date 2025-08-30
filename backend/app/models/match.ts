import { DateTime } from 'luxon'
import { BaseModel, column } from '@adonisjs/lucid/orm'

export default class Match extends BaseModel {
  @column({ isPrimary: true })
  declare id: number

  @column()
  declare status: string

  @column({
    prepare: (value: number[]) => JSON.stringify(value),
    consume: (value: string) => JSON.parse(value),
  })
  declare playerIds: number[]

  @column()
  declare serverPort: number | null

  @column()
  declare authToken: string | null

  @column.dateTime({ autoCreate: true })
  declare createdAt: DateTime

  @column.dateTime({ autoCreate: true, autoUpdate: true })
  declare updatedAt: DateTime
}