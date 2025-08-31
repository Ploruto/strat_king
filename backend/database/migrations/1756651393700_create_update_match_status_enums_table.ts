import { BaseSchema } from '@adonisjs/lucid/schema'

export default class extends BaseSchema {
  protected tableName = 'matches'

  async up() {
    // SQLite doesn't support altering enum types directly
    // So we need to recreate the table with the new enum values
    
    // First, create a temporary table with new enum
    this.schema.createTable('matches_new', (table) => {
      table.increments('id')
      table.json('player_ids').notNullable()
      table.enum('status', ['pending', 'active', 'completed', 'spawning', 'failed']).notNullable().defaultTo('pending')
      table.integer('server_port').nullable()
      table.text('auth_token').nullable()
      table.string('server_secret').notNullable()

      table.timestamp('created_at')
      table.timestamp('updated_at')
    })

    // Copy data from old table to new table
    this.defer(async (db) => {
      const matches = await db.from('matches').select('*')
      if (matches.length > 0) {
        await db.table('matches_new').insert(matches)
      }
    })

    // Drop old table and rename new table
    this.schema.dropTable('matches')
    this.schema.renameTable('matches_new', 'matches')
  }

  async down() {
    // Revert back to original enum values
    this.schema.createTable('matches_old', (table) => {
      table.increments('id')
      table.json('player_ids').notNullable()
      table.enum('status', ['pending', 'active', 'completed']).notNullable().defaultTo('pending')
      table.integer('server_port').nullable()
      table.text('auth_token').nullable()
      table.string('server_secret').notNullable()

      table.timestamp('created_at')
      table.timestamp('updated_at')
    })

    this.defer(async (db) => {
      const matches = await db.from('matches').select('*')
      if (matches.length > 0) {
        await db.table('matches_old').insert(matches)
      }
    })

    this.schema.dropTable('matches')
    this.schema.renameTable('matches_old', 'matches')
  }
}