export default async () => {
  throw new Error(
    'The bundled Muya Rust editor is disabled. Build with ELEPHANT_EXPERIMENTAL_RUST_EDITOR=1.'
  )
}

export class MuyaEditor {
  constructor() {
    throw new Error('The bundled Muya Rust editor is disabled in this build.')
  }
}
