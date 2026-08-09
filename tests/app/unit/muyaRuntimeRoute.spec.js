import { describe, expect, it } from 'vitest'

import routes from '../../../Elephant/frontend/src/renderer/src/router/index.js'

describe('Muya runtime route', () => {
  it('exposes the real app runtime test page', () => {
    const routeList = routes('editor')
    const route = routeList.find((item) => item.path === '/muya-runtime-test')
    expect(route).toBeTruthy()
    expect(route.component).toBeTruthy()
  })
})
