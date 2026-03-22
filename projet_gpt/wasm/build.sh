#!/bin/bash
# WASM build script for C Editor

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🌐 Building C Editor for WebAssembly${NC}"

# Check if Emscripten is installed
if ! command -v emcc &> /dev/null; then
    echo -e "${RED}❌ Emscripten not found. Please install Emscripten SDK.${NC}"
    echo "Visit: https://emscripten.org/docs/getting_started/downloads.html"
    exit 1
fi

echo -e "${YELLOW}📋 Emscripten version:${NC}"
emcc --version | head -1

# Create output directory
mkdir -p ../flutter/web

echo -e "${YELLOW}🔨 Compiling C sources...${NC}"

# Source files
SOURCES=(
    "../src/editor.c"
    "../src/markdown.c"
    "../src/json.c"
    "../src/editor_abi.c"
)

# Export functions for WASM
EXPORTED_FUNCTIONS=(
    "_malloc"
    "_free"
    "_editor_library_init"
    "_editor_library_cleanup"
    "_editor_parse_markdown"
    "_editor_export_markdown"
    "_editor_state_create"
    "_editor_state_destroy"
    "_editor_state_input_char"
    "_editor_state_input_string"
    "_editor_state_get_document"
    "_editor_state_get_markdown"
    "_editor_free_string"
    "_editor_get_error_message"
    "_editor_get_version_string"
)

# Join exported functions with commas
EXPORTED_FUNCS=$(IFS=,; echo "${EXPORTED_FUNCTIONS[*]}")

# Compiler flags
FLAGS=(
    "-O3"                                    # Optimize for size and speed
    "-std=c11"                              # C11 standard
    "-Wall"                                 # Enable warnings
    "-Wextra"                              # Extra warnings
    "-DWASM_BUILD=1"                       # Define WASM build
    "-DNDEBUG"                             # Disable debug assertions
    "-s" "WASM=1"                          # Generate WASM output
    "-s" "EXPORTED_FUNCTIONS=[${EXPORTED_FUNCS}]"  # Export functions
    "-s" "EXPORTED_RUNTIME_METHODS=[\"ccall\",\"cwrap\",\"UTF8ToString\",\"stringToUTF8\",\"lengthBytesUTF8\"]"
    "-s" "ALLOW_MEMORY_GROWTH=1"           # Allow memory to grow
    "-s" "INITIAL_MEMORY=2MB"              # Initial memory size
    "-s" "MAXIMUM_MEMORY=32MB"             # Maximum memory size
    "-s" "STACK_SIZE=1MB"                  # Stack size
    "-s" "NO_EXIT_RUNTIME=1"               # Don't exit runtime
    "-s" "MODULARIZE=1"                    # Use modular output
    "-s" "EXPORT_NAME=EditorModule"        # Module name
    "-s" "ENVIRONMENT=web"                 # Web environment only
    "-s" "FILESYSTEM=0"                    # Disable filesystem
    "-s" "FETCH=0"                         # Disable fetch
    "-s" "TEXTDECODER=2"                   # Use TextDecoder polyfill
    "-s" "SUPPORT_LONGJMP=0"               # Disable longjmp
    "--no-entry"                           # No main entry point
)

# Build command
echo -e "${YELLOW}⚙️  Building with flags:${NC}"
echo "  Sources: ${SOURCES[*]}"
echo "  Output: ../flutter/web/editor.js"
echo "  Exported functions: $(echo ${EXPORTED_FUNCS} | tr ',' '\n' | wc -l) functions"

emcc "${SOURCES[@]}" "${FLAGS[@]}" -o "../flutter/web/editor.js"

# Check if build was successful
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ WASM build completed successfully!${NC}"
    
    # Show file sizes
    echo -e "${BLUE}📊 Output files:${NC}"
    ls -lh ../flutter/web/editor.*
    
    # Verify WASM module
    if command -v wasm-validate &> /dev/null; then
        echo -e "${YELLOW}🔍 Validating WASM module...${NC}"
        if wasm-validate ../flutter/web/editor.wasm; then
            echo -e "${GREEN}✅ WASM module is valid${NC}"
        else
            echo -e "${RED}❌ WASM module validation failed${NC}"
        fi
    fi
    
else
    echo -e "${RED}❌ WASM build failed!${NC}"
    exit 1
fi

echo -e "${BLUE}🎉 WASM build complete! Ready for Flutter web deployment.${NC}"