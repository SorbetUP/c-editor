import { h } from '../snabbdom'

export const renderExecutableRunButton = (block) => h('button.en-code-native-run', {
  attrs: {
    type: 'button',
    title: 'Run code block',
    'aria-label': 'Run code block',
    contenteditable: 'false',
    spellcheck: 'false'
  },
  dataset: {
    blockKey: block.key
  }
}, h('span.en-code-native-run-icon'))

export const renderExecutableOutput = (block) => h('elephant-code-output.en-code-native-output', {
  attrs: {
    contenteditable: 'false',
    spellcheck: 'false'
  },
  dataset: {
    blockKey: block.key
  }
})
