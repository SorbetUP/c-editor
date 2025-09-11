#!/bin/bash
set -e

echo "🌐 Building C Editor for WebAssembly"

# Check if emscripten is available
if ! command -v emcc &> /dev/null; then
    echo "❌ Emscripten not found. Please install and activate emsdk."
    exit 1
fi

echo "📋 Emscripten version:"
emcc --version

echo "🔨 Compiling C sources..."

# List of C source files
SOURCES="../src/editor.c ../src/markdown.c ../src/json.c ../src/editor_abi.c"

# Output files
OUTPUT_JS="../docs/editor.js"
OUTPUT_WASM="../docs/editor.wasm"

echo "⚙️  Building with flags:"
echo "  Sources: $SOURCES"
echo "  Output: $OUTPUT_JS"

# Emscripten compilation flags
EMCC_FLAGS=(
    -O3
    -s WASM=1
    -s EXPORTED_RUNTIME_METHODS='["ccall","cwrap"]'
    -s EXPORTED_FUNCTIONS='["_malloc","_free","_editor_library_init","_editor_library_cleanup","_editor_get_version_string","_editor_parse_markdown","_editor_parse_markdown_simple","_editor_export_markdown","_editor_state_create","_editor_state_destroy","_editor_state_input_char","_editor_state_input_string","_editor_state_get_document","_editor_state_get_markdown","_editor_free_string","_editor_get_error_message"]'
    -s ALLOW_MEMORY_GROWTH=1
    -s INITIAL_MEMORY=1MB
    -s STACK_SIZE=512KB
    -s MODULARIZE=1
    -s EXPORT_NAME='"EditorModule"'
    --no-entry
)

echo "  Exported functions:       15 functions"

# Compile
emcc $SOURCES "${EMCC_FLAGS[@]}" -o "$OUTPUT_JS"

echo "✅ WASM build completed successfully!"

echo "📊 Output files:"
ls -lh "$OUTPUT_JS" "$OUTPUT_WASM"

echo "🎉 WASM build complete! Ready for Flutter web deployment."