import vine from '@vinejs/vine'

export const loginValidator = vine.compile(
  vine.object({
    username: vine.string().minLength(3).maxLength(50),
    password: vine.string().minLength(3)
  })
)

export const registerValidator = vine.compile(
  vine.object({
    username: vine.string().minLength(3).maxLength(50),
    password: vine.string().minLength(3)
  })
)