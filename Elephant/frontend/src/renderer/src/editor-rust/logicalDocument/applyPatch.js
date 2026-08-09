import { insertSubtree, moveNode, removeNode } from './structurePatches'
import { insertNode, replaceText, setBlockKind } from './textPatches'

export const applyLogicalPatch = (document, patch) => {
  if (!patch || typeof patch !== 'object') {
    throw new TypeError('Elephant Rust patch must be an object.')
  }
  switch (patch.type) {
    case 'replace_text':
      return replaceText(document, patch)
    case 'insert_node':
      return insertNode(document, patch.parent, patch.index, patch.node)
    case 'insert_subtree':
      return insertSubtree(document, patch.parent, patch.index, patch.subtree)
    case 'move_node':
      return moveNode(document, patch.node, patch.new_parent, patch.new_index)
    case 'remove_node':
      return removeNode(document, patch.node)
    case 'set_block_kind':
      return setBlockKind(document, patch.node, patch.kind)
    default:
      throw new TypeError(`Unknown Elephant Rust patch type: ${String(patch.type)}`)
  }
}
