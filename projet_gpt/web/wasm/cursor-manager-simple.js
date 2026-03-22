// Simple Cursor Management System for Hybrid Markdown Editor
// Lightweight version that integrates with existing editor

class SimpleCursorManager {
    constructor() {
        this.isTransitioning = false;
        this.debugMode = true;
    }
    
    log(message, data = null) {
        if (this.debugMode) {
            console.log(`[SimpleCursorManager] ${message}`, data || '');
        }
    }
    
    // Enhance the existing handleEnterKey function
    enhanceEnterKeyHandler(originalHandler) {
        const self = this;
        
        return function(e) {
            self.log('🎯 Enhanced Enter key called');
            
            // Check if we're in a problematic formatting situation
            const currentLine = editorLines[currentLineIndex];
            if (!currentLine) {
                self.log('⚠️ No current line, using original handler');
                return originalHandler.call(this, e);
            }
            
            const content = lineContents[currentLineIndex] || '';
            const isMarkdownMode = currentLine.classList.contains('current-line');
            
            // Get cursor position
            let cursorPosition = 0;
            try {
                const selection = window.getSelection();
                if (selection.rangeCount > 0) {
                    const range = selection.getRangeAt(0);
                    const preCaretRange = range.cloneRange();
                    preCaretRange.selectNodeContents(currentLine);
                    preCaretRange.setEnd(range.startContainer, range.startOffset);
                    cursorPosition = preCaretRange.toString().length;
                }
            } catch (error) {
                self.log('⚠️ Error getting cursor position:', error);
                return originalHandler.call(this, e);
            }
            
            self.log(`📍 Current state: line ${currentLineIndex}, pos ${cursorPosition}, content: "${content}"`);
            
            // Check for problematic situations and apply smart positioning
            if (self.isProblematicPosition(content, cursorPosition, isMarkdownMode)) {
                self.log('🎯 Problematic position detected, applying smart handling');
                
                try {
                    const adjustedPosition = self.adjustPositionForFormatting(content, cursorPosition, isMarkdownMode);
                    if (adjustedPosition !== cursorPosition) {
                        self.log(`🔧 Adjusting position from ${cursorPosition} to ${adjustedPosition}`);
                        
                        // Apply the adjustment and then proceed with original handler
                        self.setCursorPosition(currentLine, adjustedPosition);
                        
                        // Small delay to let the position settle
                        setTimeout(() => {
                            originalHandler.call(this, e);
                        }, 10);
                        return;
                    }
                } catch (error) {
                    self.log('⚠️ Error in smart positioning:', error);
                }
            }
            
            // Use original handler
            self.log('✅ Using original Enter key handler');
            return originalHandler.call(this, e);
        };
    }
    
    // Check if the current position is problematic
    isProblematicPosition(content, position, isMarkdownMode) {
        // Check if we're about to split inside formatting markers
        if (content.includes('==') && content.indexOf('==') < position && content.lastIndexOf('==') > position) {
            this.log('🎨 Detected position inside highlight markers');
            return true;
        }
        
        if (content.includes('*') && !isMarkdownMode) {
            const beforePos = content.substring(0, position);
            const afterPos = content.substring(position);
            if (beforePos.includes('*') && afterPos.includes('*')) {
                this.log('💫 Detected position inside italic markers');
                return true;
            }
        }
        
        return false;
    }
    
    // Adjust cursor position to avoid splitting within formatting markers
    adjustPositionForFormatting(content, position, isMarkdownMode) {
        // Convert HTML position to markdown position if needed
        let markdownPosition = position;
        if (!isMarkdownMode && typeof mapHtmlPositionToMarkdown === 'function') {
            try {
                markdownPosition = mapHtmlPositionToMarkdown(position, content);
                this.log(`🔄 Mapped HTML ${position} -> MD ${markdownPosition}`);
            } catch (error) {
                this.log('⚠️ Position mapping failed:', error);
            }
        }
        
        // Check for highlight markers ==
        const highlightStart = content.lastIndexOf('==', markdownPosition - 1);
        const highlightEnd = content.indexOf('==', markdownPosition);
        
        if (highlightStart !== -1 && highlightEnd !== -1 && highlightStart !== highlightEnd) {
            const beforeHighlight = content.substring(0, highlightStart);
            const highlightCount = (beforeHighlight.match(/==/g) || []).length;
            
            if (highlightCount % 2 === 0) {
                this.log(`🎨 Adjusting position to before highlight: ${highlightStart}`);
                return highlightStart;
            }
        }
        
        // Check for italic markers *
        const italicStart = content.lastIndexOf('*', markdownPosition - 1);
        const italicEnd = content.indexOf('*', markdownPosition);
        
        if (italicStart !== -1 && italicEnd !== -1 && italicStart !== italicEnd) {
            // Make sure it's not part of **
            if (content[italicStart - 1] !== '*' && content[italicEnd + 1] !== '*') {
                const beforeItalic = content.substring(0, italicStart);
                const italicCount = (beforeItalic.match(/\*/g) || []).length;
                
                if (italicCount % 2 === 0) {
                    this.log(`💫 Adjusting position to before italic: ${italicStart}`);
                    return italicStart;
                }
            }
        }
        
        return markdownPosition;
    }
    
    // Set cursor position within a line element
    setCursorPosition(lineElement, position) {
        try {
            lineElement.focus();
            
            const range = document.createRange();
            const selection = window.getSelection();
            
            const textNode = lineElement.firstChild;
            if (textNode && textNode.nodeType === Node.TEXT_NODE) {
                const maxPos = textNode.textContent.length;
                const safePosition = Math.min(position, maxPos);
                
                range.setStart(textNode, safePosition);
                range.collapse(true);
                selection.removeAllRanges();
                selection.addRange(range);
                
                this.log(`✅ Cursor set to position ${safePosition}`);
                return true;
            } else {
                this.log('⚠️ No text node found, setting to element start');
                range.setStart(lineElement, 0);
                range.collapse(true);
                selection.removeAllRanges();
                selection.addRange(range);
                return true;
            }
        } catch (error) {
            this.log('❌ Error setting cursor position:', error);
            return false;
        }
    }
}

// Export for use in main editor
if (typeof window !== 'undefined') {
    window.SimpleCursorManager = SimpleCursorManager;
}

if (typeof module !== 'undefined' && module.exports) {
    module.exports = SimpleCursorManager;
}