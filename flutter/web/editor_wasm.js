// JavaScript interop layer for WASM editor
(function() {
    'use strict';
    
    let wasmModule = null;
    let wasmMemory = null;
    
    // Load WASM module
    window.loadEditorWasm = async function() {
        try {
            const response = await fetch('editor.wasm');
            const wasmBytes = await response.arrayBuffer();
            
            const wasmImports = {
                env: {
                    // Memory allocation functions
                    malloc: function(size) {
                        // Simple malloc implementation
                        return wasmModule.exports.malloc(size);
                    },
                    free: function(ptr) {
                        wasmModule.exports.free(ptr);
                    },
                    
                    // Standard C library functions
                    memcpy: function(dest, src, n) {
                        const memory = new Uint8Array(wasmModule.exports.memory.buffer);
                        memory.copyWithin(dest, src, src + n);
                        return dest;
                    },
                    memset: function(ptr, value, n) {
                        const memory = new Uint8Array(wasmModule.exports.memory.buffer);
                        memory.fill(value, ptr, ptr + n);
                        return ptr;
                    },
                    strlen: function(ptr) {
                        const memory = new Uint8Array(wasmModule.exports.memory.buffer);
                        let len = 0;
                        while (memory[ptr + len] !== 0) len++;
                        return len;
                    },
                    
                    // Console output for debugging
                    printf: function(format, ...args) {
                        console.log('WASM printf:', readCString(format), args);
                        return 0;
                    },
                    puts: function(str) {
                        console.log('WASM puts:', readCString(str));
                        return 0;
                    }
                }
            };
            
            const wasmObj = await WebAssembly.instantiate(wasmBytes, wasmImports);
            wasmModule = wasmObj.instance;
            wasmMemory = wasmModule.exports.memory;
            
            // Create JavaScript interface
            const editorWasm = {
                parseMarkdown: function(markdown) {
                    try {
                        const markdownPtr = writeCString(markdown);
                        const jsonPtrPtr = wasmModule.exports.malloc(4); // pointer size
                        
                        const result = wasmModule.exports.markdown_to_json(markdownPtr, jsonPtrPtr);
                        
                        if (result !== 0) {
                            wasmModule.exports.free(markdownPtr);
                            wasmModule.exports.free(jsonPtrPtr);
                            throw new Error(`Parse failed with code ${result}`);
                        }
                        
                        // Read result pointer
                        const memory = new Uint32Array(wasmMemory.buffer);
                        const jsonPtr = memory[jsonPtrPtr / 4];
                        
                        if (jsonPtr === 0) {
                            wasmModule.exports.free(markdownPtr);
                            wasmModule.exports.free(jsonPtrPtr);
                            throw new Error('No JSON output received');
                        }
                        
                        const jsonString = readCString(jsonPtr);
                        
                        // Clean up
                        wasmModule.exports.free(markdownPtr);
                        wasmModule.exports.free(jsonPtr);
                        wasmModule.exports.free(jsonPtrPtr);
                        
                        return jsonString;
                        
                    } catch (e) {
                        throw new Error(`WASM parseMarkdown failed: ${e.message}`);
                    }
                },
                
                jsonToMarkdown: function(jsonString) {
                    try {
                        const jsonPtr = writeCString(jsonString);
                        const markdownPtrPtr = wasmModule.exports.malloc(4);
                        
                        const result = wasmModule.exports.json_to_markdown(jsonPtr, markdownPtrPtr);
                        
                        if (result !== 0) {
                            wasmModule.exports.free(jsonPtr);
                            wasmModule.exports.free(markdownPtrPtr);
                            throw new Error(`Export failed with code ${result}`);
                        }
                        
                        // Read result pointer
                        const memory = new Uint32Array(wasmMemory.buffer);
                        const markdownPtr = memory[markdownPtrPtr / 4];
                        
                        if (markdownPtr === 0) {
                            wasmModule.exports.free(jsonPtr);
                            wasmModule.exports.free(markdownPtrPtr);
                            throw new Error('No markdown output received');
                        }
                        
                        const markdown = readCString(markdownPtr);
                        
                        // Clean up
                        wasmModule.exports.free(jsonPtr);
                        wasmModule.exports.free(markdownPtr);
                        wasmModule.exports.free(markdownPtrPtr);
                        
                        return markdown;
                        
                    } catch (e) {
                        throw new Error(`WASM jsonToMarkdown failed: ${e.message}`);
                    }
                },
                
                editorInit: function() {
                    try {
                        const editorPtr = wasmModule.exports.editor_init();
                        if (editorPtr === 0) {
                            throw new Error('Failed to initialize editor');
                        }
                        return editorPtr;
                    } catch (e) {
                        throw new Error(`WASM editorInit failed: ${e.message}`);
                    }
                },
                
                editorInput: function(editorPtr, charCode) {
                    try {
                        wasmModule.exports.editor_input(editorPtr, charCode);
                    } catch (e) {
                        throw new Error(`WASM editorInput failed: ${e.message}`);
                    }
                },
                
                editorGetDocument: function(editorPtr) {
                    try {
                        const jsonPtrPtr = wasmModule.exports.malloc(4);
                        
                        const result = wasmModule.exports.editor_get_document(editorPtr, jsonPtrPtr);
                        
                        if (result !== 0) {
                            wasmModule.exports.free(jsonPtrPtr);
                            throw new Error(`Get document failed with code ${result}`);
                        }
                        
                        // Read result pointer
                        const memory = new Uint32Array(wasmMemory.buffer);
                        const jsonPtr = memory[jsonPtrPtr / 4];
                        
                        if (jsonPtr === 0) {
                            wasmModule.exports.free(jsonPtrPtr);
                            throw new Error('No document output received');
                        }
                        
                        const jsonString = readCString(jsonPtr);
                        
                        // Clean up
                        wasmModule.exports.free(jsonPtr);
                        wasmModule.exports.free(jsonPtrPtr);
                        
                        return jsonString;
                        
                    } catch (e) {
                        throw new Error(`WASM editorGetDocument failed: ${e.message}`);
                    }
                },
                
                editorFree: function(editorPtr) {
                    try {
                        wasmModule.exports.editor_free(editorPtr);
                    } catch (e) {
                        throw new Error(`WASM editorFree failed: ${e.message}`);
                    }
                }
            };
            
            // Make available globally
            window.EditorWasm = editorWasm;
            
            // Notify Dart
            if (window._editorWasmCallback) {
                window._editorWasmCallback(editorWasm);
            }
            
        } catch (error) {
            console.error('Failed to load WASM:', error);
            if (window._editorWasmErrorCallback) {
                window._editorWasmErrorCallback(error.message);
            }
            throw error;
        }
    };
    
    // Helper functions
    function writeCString(str) {
        const encoder = new TextEncoder();
        const bytes = encoder.encode(str + '\0');
        const ptr = wasmModule.exports.malloc(bytes.length);
        const memory = new Uint8Array(wasmMemory.buffer);
        memory.set(bytes, ptr);
        return ptr;
    }
    
    function readCString(ptr) {
        const memory = new Uint8Array(wasmMemory.buffer);
        let end = ptr;
        while (memory[end] !== 0) end++;
        const bytes = memory.subarray(ptr, end);
        const decoder = new TextDecoder();
        return decoder.decode(bytes);
    }
    
})();