export const blockTypeCases = [
  ['paragraph to blockquote', 'alpha', 'alpha', 'blockquote', '> alpha\n', { type: 'toggle_block_quote' }],
  ['blockquote to paragraph', '> alpha', 'alpha', 'blockquote', 'alpha\n', { type: 'toggle_block_quote' }],
  ['paragraph to bullet list', 'alpha', 'alpha', 'ul-bullet', '- alpha\n', { type: 'set_list_kind', kind: 'unordered' }],
  ['paragraph to task list', 'alpha', 'alpha', 'ul-task', '- [ ] alpha\n', { type: 'set_list_kind', kind: 'task' }],
  ['paragraph to ordered list', 'alpha', 'alpha', 'ol-order', '1. alpha\n', { type: 'set_list_kind', kind: 'ordered' }],
  ['bullet list to paragraph', '- alpha', 'alpha', 'ul-bullet', 'alpha\n', { type: 'set_list_kind', kind: 'unordered' }],
  ['task list to paragraph', '- [x] alpha', 'alpha', 'ul-task', 'alpha\n', { type: 'set_list_kind', kind: 'task' }],
  ['ordered list to paragraph', '1. alpha', 'alpha', 'ol-order', 'alpha\n', { type: 'set_list_kind', kind: 'ordered' }],
  ['bullet list to task list', '- alpha', 'alpha', 'ul-task', '- [ ] alpha\n', { type: 'set_list_kind', kind: 'task' }],
  ['task list to ordered list', '- [x] alpha', 'alpha', 'ol-order', '1. alpha\n', { type: 'set_list_kind', kind: 'ordered' }],
  ['ordered list to bullet list', '1. alpha', 'alpha', 'ul-bullet', '- alpha\n', { type: 'set_list_kind', kind: 'unordered' }],
  ['paragraph to fenced code', 'alpha', 'alpha', 'pre', '```\nalpha\n```\n', { type: 'toggle_code_block' }],
  ['fenced code back to paragraph', '```\nalpha\n```', 'alpha', 'pre', 'alpha\n', { type: 'toggle_code_block' }, true]
]

export const blockTypeHistoryCases = [
  {
    name: 'undo and redo a blockquote toggle',
    initial: 'alpha',
    target: 'alpha',
    paragraphType: 'blockquote',
    command: { type: 'toggle_block_quote' },
    checkpoints: ['> alpha\n', 'alpha\n', '> alpha\n']
  },
  {
    name: 'undo and redo a bullet-list wrapper',
    initial: 'alpha',
    target: 'alpha',
    paragraphType: 'ul-bullet',
    command: { type: 'set_list_kind', kind: 'unordered' },
    checkpoints: ['- alpha\n', 'alpha\n', '- alpha\n']
  },
  {
    name: 'undo and redo a fenced-code toggle',
    initial: 'alpha',
    target: 'alpha',
    paragraphType: 'pre',
    command: { type: 'toggle_code_block' },
    checkpoints: ['```\nalpha\n```\n', 'alpha\n', '```\nalpha\n```\n']
  }
]
