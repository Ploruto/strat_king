/*
|--------------------------------------------------------------------------
| Routes file
|--------------------------------------------------------------------------
|
| The routes file is used for defining the HTTP routes.
|
*/

import router from '@adonisjs/core/services/router'

router.get('/', async () => {
  return {
    hello: 'world',
  }
})

router.group(() => {
  router.post('/login', '#controllers/auth_controller.login')
  router.post('/register', '#controllers/auth_controller.register')
}).prefix('/auth')

router.group(() => {
  router.post('/join', '#controllers/matchmaking_controller.join').middleware('auth')
  router.delete('/leave', '#controllers/matchmaking_controller.leave').middleware('auth')
  router.get('/ws', '#controllers/matchmaking_websocket_controller.connect')
}).prefix('/matchmaking')
