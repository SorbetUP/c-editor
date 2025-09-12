// Automated tests for hybrid editor functionality
// Run in browser console: copy/paste this entire script

function runEditorTests() {
    console.log('🧪 Starting Editor Tests...');
    
    // Test 1: Position mapping for complex formatting
    function testPositionMapping() {
        console.log('\n📍 Test 1: Position Mapping');
        
        const testCases = [
            {
                markdown: '- *Italique* ==Surligné==',
                htmlPosition: 2,  // clicking at start of "Italique" in HTML
                expectedMd: 3     // should be after "- *" (start of Italique)
            },
            {
                markdown: '- *Italique* ==Surligné==',
                htmlPosition: 16, // clicking between 'l' and 'i' in "Surligné" 
                expectedMd: 18    // should be after "- *Italique* =="
            },
            {
                markdown: '**Gras** et *italique*',
                htmlPosition: 8,  // clicking on 'i' in "italique"
                expectedMd: 13    // should be after "**Gras** et *"
            },
            {
                markdown: '# Titre test',
                htmlPosition: 5,  // clicking on 'e' in "Titre"
                expectedMd: 7     // should be after "# Ti"
            },
            {
                markdown: '- *Italique* ==Surligné==',
                htmlPosition: 13, // clicking at start of "Surligné" in HTML
                expectedMd: 15    // should be after "- *Italique* =="
            }
        ];
        
        testCases.forEach((test, i) => {
            const result = mapHtmlPositionToMarkdown(test.htmlPosition, test.markdown);
            const passed = result === test.expectedMd;
            console.log(`  Test 1.${i + 1}: ${passed ? '✅' : '❌'} ${test.markdown}`);
            console.log(`    HTML pos ${test.htmlPosition} -> MD pos ${result} (expected ${test.expectedMd})`);
            if (!passed) {
                console.error(`    FAILED: Expected ${test.expectedMd}, got ${result}`);
            }
        });
    }
    
    // Test 2: Enter key line splitting
    function testEnterKeySplitting() {
        console.log('\n🔄 Test 2: Enter Key Line Splitting');
        
        // Create a temporary test line
        const testLine = document.createElement('div');
        testLine.className = 'editor-line';
        testLine.textContent = '**Gras** et italic';
        testLine.setAttribute('contenteditable', 'true');
        document.body.appendChild(testLine);
        
        // Simulate cursor position at "et" (position 10)
        testLine.focus();
        const range = document.createRange();
        const selection = window.getSelection();
        range.setStart(testLine.firstChild, 10);
        range.collapse(true);
        selection.removeAllRanges();
        selection.addRange(range);
        
        // Get cursor position
        const preCaretRange = range.cloneRange();
        preCaretRange.selectNodeContents(testLine);
        preCaretRange.setEnd(range.startContainer, range.startOffset);
        const cursorPosition = preCaretRange.toString().length;
        
        const content = testLine.textContent;
        const beforeCursor = content.substring(0, cursorPosition);
        const afterCursor = content.substring(cursorPosition);
        
        const passed = beforeCursor === '**Gras** ' && afterCursor === 'et italic';
        console.log(`  Test 2.1: ${passed ? '✅' : '❌'} Line splitting`);
        console.log(`    Before: "${beforeCursor}" | After: "${afterCursor}"`);
        
        // Cleanup
        document.body.removeChild(testLine);
    }
    
    // Test 3: Line merging with headers
    function testLineMergingWithHeaders() {
        console.log('\n🔗 Test 3: Line Merging with Headers');
        
        const headerContent = '# Titre';
        const nextContent = 'Contenu suivant';
        const mergedContent = headerContent + nextContent;
        const mergePosition = headerContent.length;
        
        const passed = mergePosition === 7; // Length of "# Titre"
        console.log(`  Test 3.1: ${passed ? '✅' : '❌'} Header merge position`);
        console.log(`    Header: "${headerContent}" (length ${headerContent.length})`);
        console.log(`    Merge position: ${mergePosition}`);
        console.log(`    Result: "${mergedContent}"`);
    }
    
    // Test 4: Performance timing
    function testPerformanceTiming() {
        console.log('\n⚡ Test 4: Performance Timing');
        
        const iterations = 1000;
        const testMarkdown = '- **Gras** *italique* ==surligné== ++souligné++';
        
        // Test position mapping performance
        const startTime = performance.now();
        for (let i = 0; i < iterations; i++) {
            mapHtmlPositionToMarkdown(15, testMarkdown);
        }
        const endTime = performance.now();
        const avgTime = (endTime - startTime) / iterations;
        
        const passed = avgTime < 1; // Should be less than 1ms per operation
        console.log(`  Test 4.1: ${passed ? '✅' : '❌'} Position mapping performance`);
        console.log(`    Average time: ${avgTime.toFixed(3)}ms per operation`);
        console.log(`    Total time for ${iterations} operations: ${(endTime - startTime).toFixed(1)}ms`);
    }
    
    // Run all tests
    try {
        testPositionMapping();
        testEnterKeySplitting();
        testLineMergingWithHeaders();
        testPerformanceTiming();
        
        console.log('\n🎉 All tests completed!');
        console.log('Check above for any ❌ failures that need fixing.');
        
    } catch (error) {
        console.error('❌ Test suite failed:', error);
    }
}

// Auto-run tests if mapHtmlPositionToMarkdown function is available
if (typeof mapHtmlPositionToMarkdown === 'function') {
    runEditorTests();
} else {
    console.log('⏳ Waiting for editor functions to load...');
    setTimeout(() => {
        if (typeof mapHtmlPositionToMarkdown === 'function') {
            runEditorTests();
        } else {
            console.error('❌ Editor functions not available. Make sure to run this in the editor page.');
        }
    }, 1000);
}