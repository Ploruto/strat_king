import { BaseSchema } from '@adonisjs/lucid/schema'

export default class extends BaseSchema {
  protected tableName = 'matches'

  async up() {
    this.schema.createTable(this.tableName, (table) => {
      table.increments('id')
      table.json('player_ids').notNullable()
      table.enum('status', ['pending', 'active', 'completed']).notNullable().defaultTo('pending')
      table.integer('server_port').nullable()
      table.text('auth_token').nullable()

      table.timestamp('created_at')
      table.timestamp('updated_at')
    })
  }

  async down() {
    this.schema.dropTable(this.tableName)
  }
}