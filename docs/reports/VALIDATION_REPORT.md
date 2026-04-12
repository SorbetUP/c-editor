# Validation Report

**Commit SHA**: `b173d6d823198b95978047576a754157c6f48152`  
**Timestamp**: 2026-03-22 20:49:19 UTC  
**Status**: ✅ PASSED

## Checks

- ✅ Root build (make: Nothing to be done for `all'.)
- ✅ Canonical engine tests (🧪 Running canonical engine test suite...
rm -f libeditor.a libeditor.so libeditor_debug.a
rm -f editor.js editor.wasm editor_debug.wasm.*
rm -f *.o render_ext.o render_ext_debug.o editor_test
🧹 Cleaned build artifacts
clang -std=c11 -Wall -Wextra -Werror -O2 -g -DDEBUG_EDITOR=0 -DNDEBUG -I../markdown -I../render_ext -I. -c editor.c -o editor.o
clang -std=c11 -Wall -Wextra -Werror -O2 -g -DDEBUG_EDITOR=0 -DNDEBUG -I../markdown -I../render_ext -I. -c editor_abi.c -o editor_abi.o
clang -std=c11 -Wall -Wextra -Werror -O2 -g -DDEBUG_EDITOR=0 -DNDEBUG -I../markdown -I../render_ext -I. -c ../render_ext/render_ext.c -o render_ext.o
ar rcs libeditor.a editor.o editor_abi.o render_ext.o
✅ Static library built: libeditor.a
/Applications/Xcode.app/Contents/Developer/usr/bin/make -C ../markdown static
make[2]: Nothing to be done for `static'.
clang -std=c11 -Wall -Wextra -Werror -O2 -g -DDEBUG_EDITOR=1 -DDEBUG_VERBOSE=1 -I../markdown -I../render_ext -I. test.c -L. -leditor -L../markdown -lmarkdown -o editor_test
✅ Test program built: editor_test
Run with: ./editor_test
editor tests passed
rm -f libmarkdown.a libmarkdown.so libmarkdown_debug.a
rm -f markdown.wasm.js markdown.wasm markdown_debug.wasm.*
rm -f *.o markdown_test
🧹 Cleaned build artifacts
clang -std=c11 -Wall -Wextra -Werror -O2 -g -DDEBUG_MARKDOWN=0 -DNDEBUG -I../editor -c markdown.c -o markdown.o
clang -std=c11 -Wall -Wextra -Werror -O2 -g -DDEBUG_MARKDOWN=0 -DNDEBUG -I../editor -c json.c -o json.o
ar rcs libmarkdown.a markdown.o json.o
✅ Static library built: libmarkdown.a
/Applications/Xcode.app/Contents/Developer/usr/bin/make -C ../editor static
make[2]: Nothing to be done for `static'.
clang -std=c11 -Wall -Wextra -Werror -O2 -g -DDEBUG_MARKDOWN=1 -DDEBUG_VERBOSE=1 -I../editor test.c -L. -lmarkdown -L../editor -leditor -o markdown_test
✅ Test program built: markdown_test
Run with: ./markdown_test
✅ list marker tests passed
✅ markdown table tests passed
rm -f libcursor.a libcursor.so libcursor_debug.a
rm -f cursor_wasm.js cursor_wasm.wasm cursor_debug.wasm.*
rm -f *.o test_cursor cursor_test_demo benchmark
🧹 Cleaned build artifacts
clang -std=c11 -Wall -Wextra -Werror -O2 -g -DDEBUG_CURSOR=0 -DNDEBUG -c cursor_manager.c -o cursor_manager.o
ar rcs libcursor.a cursor_manager.o
✅ Static library built: libcursor.a
clang -std=c11 -Wall -Wextra -Werror -O2 -g -DDEBUG_CURSOR=1 -DDEBUG_VERBOSE=1 test_cursor.c -L. -lcursor -o test_cursor
✅ Test program built: test_cursor
./test_cursor
cursor tests passed
rm -f crypto_engine.o libcrypto_engine.a test_crypto
clang -std=c11 -Wall -Wextra -Werror -O2 -g -DNDEBUG -DDEBUG_CRYPTO=0 -c crypto_engine.c -o crypto_engine.o
ar rcs libcrypto_engine.a crypto_engine.o
✅ Static library built: libcrypto_engine.a
clang -std=c11 -Wall -Wextra -Werror -O2 -g -DDEBUG_CRYPTO=0 test_crypto.c libcrypto_engine.a -o test_crypto
./test_crypto
crypto_engine tests passed
✅ Canonical tests passed)
- ✅ GitHub Pages build (==> Building GitHub Pages artifact
Root: /Users/sorbet/Desktop/Dev/c-editor
rm -f libeditor.a libeditor.so libeditor_debug.a
rm -f editor.js editor.wasm editor_debug.wasm.*
rm -f *.o render_ext.o render_ext_debug.o editor_test
🧹 Cleaned build artifacts
emcc -std=c11 -O2 -s WASM=1 -s EXPORTED_RUNTIME_METHODS='["cwrap","ccall"]' -s ALLOW_MEMORY_GROWTH=1 -s MODULARIZE=1 -s EXPORT_NAME="EditorModule" --no-entry -s EXPORTED_FUNCTIONS='["_malloc","_free","_editor_library_init","_editor_library_cleanup","_editor_get_version_string","_editor_parse_markdown","_editor_parse_markdown_simple","_editor_markdown_to_html","_editor_export_markdown","_editor_state_create","_editor_state_destroy","_editor_state_reset","_editor_state_input_char","_editor_state_input_string","_editor_state_backspace","_editor_state_delete","_editor_state_get_document","_editor_state_get_markdown","_editor_free_string","_editor_get_error_message","_editor_enable_debug_logging"]' -DDEBUG_EDITOR=0 -DNDEBUG -I../markdown -I../render_ext -I. editor.c editor_abi.c ../render_ext/render_ext.c ../markdown/markdown.c ../markdown/json.c -o editor.js
✅ WebAssembly module built: editor.js
rm -f libcursor.a libcursor.so libcursor_debug.a
rm -f cursor_wasm.js cursor_wasm.wasm cursor_debug.wasm.*
rm -f *.o test_cursor cursor_test_demo benchmark
🧹 Cleaned build artifacts
emcc -std=c11 -O2 -s WASM=1 -s EXPORTED_RUNTIME_METHODS='["cwrap","ccall"]' -s ALLOW_MEMORY_GROWTH=1 -s MODULARIZE=1 -s EXPORT_NAME="CursorModule" --no-entry -s EXPORTED_FUNCTIONS='["_malloc","_free","_cursor_wasm_html_to_markdown","_cursor_wasm_adjust_for_formatting","_cursor_wasm_is_inside_formatting","_cursor_wasm_get_formatting_type","_cursor_wasm_handle_enter_key","_cursor_wasm_split_line","_cursor_wasm_merge_lines","_cursor_wasm_validate_position","_cursor_wasm_find_safe_position","_cursor_wasm_free","_cursor_wasm_debug"]' -DDEBUG_CURSOR=0 -DNDEBUG cursor_manager.c cursor_wasm.c -o cursor_wasm.js
✅ WebAssembly module built: cursor_wasm.js
==> GitHub Pages artifact ready
-rw-r--r--@ 1 sorbet  staff    13K Mar 22 21:49 /Users/sorbet/Desktop/Dev/c-editor/dist/github-pages/docs/cursor_wasm.js
-rwxr-xr-x@ 1 sorbet  staff    21K Mar 22 21:49 /Users/sorbet/Desktop/Dev/c-editor/dist/github-pages/docs/cursor_wasm.wasm
-rw-r--r--@ 1 sorbet  staff    63K Mar 22 21:49 /Users/sorbet/Desktop/Dev/c-editor/dist/github-pages/docs/editor.js
-rwxr-xr-x@ 1 sorbet  staff   110K Mar 22 21:49 /Users/sorbet/Desktop/Dev/c-editor/dist/github-pages/docs/editor.wasm)
- ✅ GitHub Pages smoke test (==> Smoke testing GitHub Pages artifact
==> GitHub Pages smoke test passed)
