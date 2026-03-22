// JavaScript interface for C cursor management library
// Provides high-level API for cursor operations backed by WebAssembly C code

class CCursorManager {
    constructor(wasmModule) {
        this.module = wasmModule;
        this.isReady = false;
        
        if (wasmModule) {
            this.initializeBindings();
        }
    }
    
    initializeBindings() {
        // Check if ccall and cwrap are available for calling C functions
        if (typeof this.module.ccall !== 'function' || typeof this.module.cwrap !== 'function') {
            console.error('[CCursorManager] Missing ccall/cwrap functions');
            return;
        }
        
        try {
            // Test if we can call at least one function
            const testResult = this.module.ccall('cursor_wasm_validate_position', 'number', ['string', 'number'], ['test', 0]);
            this.isReady = true;
            console.log('[CCursorManager] ✅ C cursor management library ready');
        } catch (error) {
            console.error('[CCursorManager] ❌ Failed to initialize:', error);
        }
    }
    
    // Convert HTML position to Markdown position
    htmlToMarkdown(htmlPosition, markdownText) {
        if (!this.isReady) {
            console.warn('[CCursorManager] Not ready, using fallback');
            return htmlPosition; // Fallback
        }
        
        try {
            const result = this.module.ccall('cursor_wasm_html_to_markdown', 'number', ['number', 'string'], [htmlPosition, markdownText]);
            console.log(`[CCursorManager] 🔄 HTML ${htmlPosition} -> MD ${result}`);
            return result >= 0 ? result : htmlPosition;
        } catch (error) {
            console.error('[CCursorManager] Error in htmlToMarkdown:', error);
            return htmlPosition;
        }
    }
    
    // Adjust position to avoid formatting conflicts
    adjustForFormatting(position, content) {
        if (!this.isReady) {
            return position;
        }
        
        try {
            const adjusted = this.module.ccall('cursor_wasm_adjust_for_formatting', 'number', ['number', 'string'], [position, content]);
            if (adjusted !== position) {
                console.log(`[CCursorManager] 🔧 Position adjusted: ${position} -> ${adjusted}`);
            }
            return adjusted;
        } catch (error) {
            console.error('[CCursorManager] Error in adjustForFormatting:', error);
            return position;
        }
    }
    
    // Check if position is inside formatting markers
    isInsideFormatting(content, position) {
        if (!this.isReady) {
            return false;
        }
        
        try {
            const result = this.module.ccall('cursor_wasm_is_inside_formatting', 'number', ['string', 'number'], [content, position]);
            return result === 1;
        } catch (error) {
            console.error('[CCursorManager] Error in isInsideFormatting:', error);
            return false;
        }
    }
    
    // Get formatting type at position
    getFormattingType(content, position) {
        if (!this.isReady) {
            return 0; // MARKER_NONE
        }
        
        try {
            return this.module.ccall('cursor_wasm_get_formatting_type', 'number', ['string', 'number'], [content, position]);
        } catch (error) {
            console.error('[CCursorManager] Error in getFormattingType:', error);
            return 0;
        }
    }
    
    // Handle Enter key press
    handleEnterKey(position, content) {
        if (!this.isReady) {
            return { success: false, error: 'C cursor manager not ready' };
        }
        
        try {
            console.log(`[CCursorManager] 🎯 Handling Enter key at position ${position}`);
            
            const jsonStr = this.module.ccall('cursor_wasm_handle_enter_key', 'string', ['number', 'string'], [position, content]);
            if (!jsonStr) {
                return { success: false, error: 'Failed to get result from C function' };
            }
            
            const result = JSON.parse(jsonStr);
            // Note: Emscripten handles string memory management automatically for return values
            
            if (result.success) {
                console.log(`[CCursorManager] ✅ Enter key handled: "${result.beforeCursor}" | "${result.afterCursor}"`);
            } else {
                console.log(`[CCursorManager] ❌ Enter key failed: ${result.error}`);
            }
            
            return result;
        } catch (error) {
            console.error('[CCursorManager] Error in handleEnterKey:', error);
            return { success: false, error: error.message };
        }
    }
    
    // Split line at position
    splitLine(position, content) {
        if (!this.isReady) {
            return { success: false, error: 'C cursor manager not ready' };
        }
        
        try {
            const jsonStr = this.module.ccall('cursor_wasm_split_line', 'string', ['number', 'string'], [position, content]);
            if (!jsonStr) {
                return { success: false, error: 'Failed to get result from C function' };
            }
            
            const result = JSON.parse(jsonStr);
            
            return result;
        } catch (error) {
            console.error('[CCursorManager] Error in splitLine:', error);
            return { success: false, error: error.message };
        }
    }
    
    // Merge two lines
    mergeLines(line1, line2, addSpace = true) {
        if (!this.isReady) {
            return { success: false, error: 'C cursor manager not ready' };
        }
        
        try {
            const jsonStr = this.module.ccall('cursor_wasm_merge_lines', 'string', ['string', 'string', 'number'], [line1, line2, addSpace ? 1 : 0]);
            if (!jsonStr) {
                return { success: false, error: 'Failed to get result from C function' };
            }
            
            const result = JSON.parse(jsonStr);
            
            if (result.success) {
                console.log(`[CCursorManager] 🔗 Lines merged: "${result.mergedContent}" (cursor at ${result.cursorPosition})`);
            }
            
            return result;
        } catch (error) {
            console.error('[CCursorManager] Error in mergeLines:', error);
            return { success: false, error: error.message };
        }
    }
    
    // Validate position
    validatePosition(content, position) {
        if (!this.isReady) {
            return position >= 0 && position <= content.length;
        }
        
        try {
            return this.module.ccall('cursor_wasm_validate_position', 'number', ['string', 'number'], [content, position]) === 1;
        } catch (error) {
            console.error('[CCursorManager] Error in validatePosition:', error);
            return false;
        }
    }
    
    // Find safe split position
    findSafePosition(content, position) {
        if (!this.isReady) {
            return position;
        }
        
        try {
            return this.module.ccall('cursor_wasm_find_safe_position', 'number', ['string', 'number'], [content, position]);
        } catch (error) {
            console.error('[CCursorManager] Error in findSafePosition:', error);
            return position;
        }
    }
    
    // Debug function
    debug(content, position) {
        if (!this.isReady) {
            console.log(`[CCursorManager] Debug (fallback): position ${position} in "${content}"`);
            return;
        }
        
        try {
            this.module.ccall('cursor_wasm_debug', 'void', ['string', 'number'], [content, position]);
        } catch (error) {
            console.error('[CCursorManager] Error in debug:', error);
        }
    }
    
    // Enhanced Enter key handler that integrates with existing editor
    createEnhancedEnterHandler(originalHandler) {
        const self = this;
        
        return function(e) {
            console.log('[CCursorManager] 🎯 Enhanced Enter key (C-powered) called');
            
            try {
                // Get current editor state
                const currentLine = editorLines[currentLineIndex];
                if (!currentLine) {
                    console.log('[CCursorManager] ⚠️ No current line, using original handler');
                    return originalHandler.call(this, e);
                }
                
                const content = lineContents[currentLineIndex] || '';
                const isMarkdownMode = currentLine.classList.contains('current-line');
                
                // Get cursor position
                let cursorPosition = 0;
                const selection = window.getSelection();
                if (selection.rangeCount > 0) {
                    const range = selection.getRangeAt(0);
                    const preCaretRange = range.cloneRange();
                    preCaretRange.selectNodeContents(currentLine);
                    preCaretRange.setEnd(range.startContainer, range.startOffset);
                    cursorPosition = preCaretRange.toString().length;
                }
                
                // Convert HTML position to markdown if needed
                let finalPosition = cursorPosition;
                if (!isMarkdownMode) {
                    finalPosition = self.htmlToMarkdown(cursorPosition, content);
                }
                
                console.log(`[CCursorManager] 📍 Current: line ${currentLineIndex}, HTML pos ${cursorPosition}, MD pos ${finalPosition}`);
                
                // Check if position needs adjustment for formatting
                const adjustedPosition = self.adjustForFormatting(finalPosition, content);
                
                if (adjustedPosition !== finalPosition) {
                    console.log(`[CCursorManager] 🔧 C library adjusted position: ${finalPosition} -> ${adjustedPosition}`);
                    
                    // Set the adjusted position before proceeding
                    const textNode = currentLine.firstChild;
                    if (textNode && textNode.nodeType === Node.TEXT_NODE) {
                        const range = document.createRange();
                        const selection = window.getSelection();
                        const safePos = Math.min(adjustedPosition, textNode.textContent.length);
                        
                        range.setStart(textNode, safePos);
                        range.collapse(true);
                        selection.removeAllRanges();
                        selection.addRange(range);
                        
                        console.log(`[CCursorManager] ✅ Cursor repositioned to ${safePos}`);
                    }
                    
                    // Small delay to let position settle
                    setTimeout(() => {
                        originalHandler.call(this, e);
                    }, 10);
                    return;
                }
                
                console.log('[CCursorManager] ✅ Position is safe, using original handler');
                
            } catch (error) {
                console.log('[CCursorManager] ⚠️ Error in enhanced handler:', error);
            }
            
            // Use original handler
            return originalHandler.call(this, e);
        };
    }
}

// Formatting type constants
CCursorManager.MARKER_NONE = 0;
CCursorManager.MARKER_BOLD = 1;
CCursorManager.MARKER_ITALIC = 2;
CCursorManager.MARKER_HIGHLIGHT = 3;
CCursorManager.MARKER_UNDERLINE = 4;
CCursorManager.MARKER_HEADER = 5;

// Global instance (will be initialized when WASM module is ready)
let cCursorManager = null;

// Initialize with WASM module
function initializeCCursorManager(wasmModule) {
    cCursorManager = new CCursorManager(wasmModule);
    console.log('[CCursorManager] 🚀 C cursor management system initialized');
    return cCursorManager;
}

// Export for different module systems
if (typeof window !== 'undefined') {
    window.CCursorManager = CCursorManager;
    window.initializeCCursorManager = initializeCCursorManager;
    window.cCursorManager = cCursorManager;
}

if (typeof module !== 'undefined' && module.exports) {
    module.exports = { CCursorManager, initializeCCursorManager };
}