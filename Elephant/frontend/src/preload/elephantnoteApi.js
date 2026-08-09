import { toPlainObject } from 'elephant-shared/plainObject'

const callIpc = (invoke, channel, payload) => invoke(channel, toPlainObject(payload))

export const createElephantNoteAPI = ({ invoke }) => ({
  api: {
    call: (action, payload = {}) =>
      callIpc(invoke, 'elephantnote:api:call', {
        action,
        payload
      })
  },
  models: {
    downloadStatus: (payload = {}) =>
      callIpc(invoke, 'elephantnote:models:download-status', payload)
  }
})
