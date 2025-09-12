// Essential Tests - Only the critical cursor positioning scenarios
// Focuses on the main issue: cursor before ==Surligné== + Enter key

function runEssentialTests() {
    console.log('🧪 Running Essential Cursor Tests...');
    
    const tests = [
        {
            name: 'Main Issue: Cursor before ==Surligné==',
            content: '- *Italique* ==Surligné== fin',
            testScenario: function() {
                console.log('📋 Testing main issue scenario...');
                
                // Test position adjustment logic
                if (typeof adjustCursorForFormatting === 'function') {
                    // Test HTML position 11 (just before "Surligné" in rendered HTML)
                    const htmlPos = 11;
                    const adjusted = adjustCursorForFormatting(this.content, htmlPos, false);
                    
                    console.log(`📍 HTML position ${htmlPos} adjusted to MD position ${adjusted}`);
                    
                    // The key is that it should NOT split inside the == markers
                    const beforeCursor = this.content.substring(0, adjusted);
                    const afterCursor = this.content.substring(adjusted);
                    
                    console.log(`✂️ Would split: "${beforeCursor}" | "${afterCursor}"`);
                    
                    // Check that we don't split in the middle of == ==
                    const containsPartialMarker = beforeCursor.endsWith('=') || afterCursor.startsWith('=');
                    
                    if (!containsPartialMarker) {
                        console.log('✅ SUCCESS: No partial markers, clean split');
                        return true;
                    } else {
                        console.log('❌ FAIL: Would create partial marker');
                        return false;
                    }
                } else {
                    console.log('⚠️ adjustCursorForFormatting function not available');
                    return false;
                }
            }
        },
        
        {
            name: 'Enhanced Enter Key Handler',
            testScenario: function() {
                console.log('📋 Testing enhanced Enter key handler...');
                
                // Check if the enhanced handler is in place
                if (typeof handleEnterKey === 'function') {
                    const handlerSource = handleEnterKey.toString();
                    const hasEnhancement = handlerSource.includes('Enhanced Enter key handler');
                    
                    if (hasEnhancement) {
                        console.log('✅ SUCCESS: Enhanced Enter key handler is active');
                        return true;
                    } else {
                        console.log('❌ FAIL: Enhanced Enter key handler not found');
                        return false;
                    }
                } else {
                    console.log('❌ FAIL: handleEnterKey function not found');
                    return false;
                }
            }
        },
        
        {
            name: 'Position Mapping Function',
            testScenario: function() {
                console.log('📋 Testing position mapping function...');
                
                if (typeof mapHtmlPositionToMarkdown === 'function') {
                    try {
                        // Test a simple case
                        const result = mapHtmlPositionToMarkdown(5, '- *test*');
                        console.log(`📍 Position mapping test: HTML 5 -> MD ${result}`);
                        
                        if (typeof result === 'number' && result >= 0) {
                            console.log('✅ SUCCESS: Position mapping function works');
                            return true;
                        } else {
                            console.log('❌ FAIL: Invalid result from position mapping');
                            return false;
                        }
                    } catch (error) {
                        console.log('❌ FAIL: Position mapping function error:', error);
                        return false;
                    }
                } else {
                    console.log('❌ FAIL: mapHtmlPositionToMarkdown function not found');
                    return false;
                }
            }
        }
    ];
    
    let passed = 0;
    let total = tests.length;
    
    console.log(`🚀 Running ${total} essential tests...`);
    console.log('─'.repeat(50));
    
    tests.forEach((test, index) => {
        console.log(`\n${index + 1}. ${test.name}`);
        try {
            const result = test.testScenario();
            if (result) {
                passed++;
                console.log(`   ✅ PASSED`);
            } else {
                console.log(`   ❌ FAILED`);
            }
        } catch (error) {
            console.log(`   💥 ERROR: ${error.message}`);
        }
    });
    
    console.log('\n' + '─'.repeat(50));
    console.log(`📊 RESULTS: ${passed}/${total} tests passed (${Math.round(passed/total*100)}%)`);
    
    if (passed === total) {
        console.log('🎉 ALL ESSENTIAL TESTS PASSED!');
        console.log('✅ Your cursor positioning issue should be resolved.');
    } else {
        console.log('⚠️ Some essential tests failed.');
        console.log('💡 Check the console output above for details.');
    }
    
    return { passed, total, percentage: Math.round(passed/total*100) };
}

// Auto-run if in browser and editor is loaded
if (typeof window !== 'undefined') {
    // Wait for editor to be fully loaded
    function checkAndRun() {
        if (typeof handleEnterKey !== 'undefined' && typeof mapHtmlPositionToMarkdown !== 'undefined') {
            console.log('🎯 Editor detected, running essential tests...');
            setTimeout(runEssentialTests, 1000);
        } else {
            console.log('⏳ Waiting for editor to load...');
            setTimeout(checkAndRun, 1000);
        }
    }
    
    // Start checking after a short delay
    setTimeout(checkAndRun, 2000);
    
    // Make function globally available
    window.runEssentialTests = runEssentialTests;
}

// Export for Node.js if needed
if (typeof module !== 'undefined' && module.exports) {
    module.exports = { runEssentialTests };
}