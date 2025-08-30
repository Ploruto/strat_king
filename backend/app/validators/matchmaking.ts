import vine from '@vinejs/vine'

export const matchmakingValidator = vine.compile(
  vine.object({
    game_mode: vine.string().in(['1v1']),
    player_id: vine.number()
  })
)