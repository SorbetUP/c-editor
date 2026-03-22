# Core WASM Assets

This directory contains the compiled WebAssembly files for the C editor core.

The WASM files are built from the C source code and provide:
- `note_md_to_json`: Convert Markdown to JSON
- `note_json_to_md`: Convert JSON to Markdown  
- `note_json_canonicalize`: Canonicalize JSON structure
- `note_version`: Get core version information

## Build Process

The WASM files are automatically built during the GitHub Pages deployment workflow using Emscripten.

For local development, you'll need to build the WASM files manually:

```bash
cd ../src
emcc -O3 -s WASM=1 -s EXPORTED_FUNCTIONS='[...]' ...
```

See `.github/workflows/deploy.yml` for the complete build command.