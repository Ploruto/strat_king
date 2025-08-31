/*
|--------------------------------------------------------------------------
| Routes file
|--------------------------------------------------------------------------
|
| The routes file is used for defining the HTTP routes.
|
*/

import router from '@adonisjs/core/services/router'
import { middleware } from '#start/kernel'
const AuthController = () => import('#controllers/auth_controller')
const MatchmakingsController = () => import('#controllers/matchmakings_controller')
const WebhooksController = () => import('#controllers/webhooks_controller')

import Match from '#models/match'

router.get('/', async () => {
  await Match.create({
        playerIds: [1, 2],
        status: 'pending'
    });

  return {
    hello: 'world',
  }
})

router.group(() => {
  router.post('/register', [AuthController, 'register'])
  router.post('/login', [AuthController, 'login'])
}).prefix('/auth')

router.post('/matchmaking/join', [MatchmakingsController, 'join'])

router.group(() => {
  router.post('/server-ready', [WebhooksController, 'serverReady'])
  router.post('/match-complete', [WebhooksController, 'matchComplete'])
}).prefix('/webhooks')
