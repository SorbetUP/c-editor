// Comprehensive validation script for position mapping
// Run this in the browser console on the editor page

function validatePositionMapping() {
    console.log('🧪 Validating Position Mapping...');
    
    // Test cases with expected mappings
    const testCases = [
        {
            name: 'Simple italic',
            markdown: '- *Italique*',
            htmlText: '- Italique',
            expectations: [
                { html: 0, md: 0, desc: 'Dash' },
                { html: 1, md: 1, desc: 'Space' },
                { html: 2, md: 3, desc: 'Start of Italique (after *)' },
                { html: 5, md: 6, desc: 'Middle of Italique' },
                { html: 9, md: 10, desc: 'End of Italique (before *)' }
            ]
        },
        {
            name: 'Italic + Highlight',
            markdown: '- *Italique* ==Surligné==',
            htmlText: '- Italique Surligné',
            expectations: [
                { html: 0, md: 0, desc: 'Dash' },
                { html: 2, md: 3, desc: 'Start of Italique' },
                { html: 11, md: 13, desc: 'Space after Italique' },
                { html: 12, md: 15, desc: 'Start of Surligné (after ==)' },
                { html: 16, md: 19, desc: 'Middle of Surligné' }
            ]
        },
        {
            name: 'Header',
            markdown: '# Titre',
            htmlText: 'Titre',
            expectations: [
                { html: 0, md: 2, desc: 'Start of Titre (after # )' },
                { html: 3, md: 5, desc: 'Middle of Titre' },
                { html: 5, md: 7, desc: 'End of Titre' }
            ]
        },
        {
            name: 'Empty line',
            markdown: '',
            htmlText: '',
            expectations: [
                { html: 0, md: 0, desc: 'Empty line start' },
                { html: 1, md: 0, desc: 'Beyond empty line' }
            ]
        },
        {
            name: 'Complex multi-format',
            markdown: '**Bold** and *italic* with ==highlight==',
            htmlText: 'Bold and italic with highlight',
            expectations: [
                { html: 0, md: 2, desc: 'Start of Bold (after **)' },
                { html: 4, md: 6, desc: 'End of Bold (before **)' },
                { html: 5, md: 9, desc: 'Space after Bold' },
                { html: 9, md: 14, desc: 'Start of italic (after *)' },
                { html: 15, md: 21, desc: 'Space after italic' },
                { html: 21, md: 29, desc: 'Start of highlight (after ==)' }
            ]
        }
    ];
    
    let totalTests = 0;
    let passedTests = 0;
    
    testCases.forEach(testCase => {
        console.log(`\n📋 Testing: ${testCase.name}`);
        console.log(`   Markdown: "${testCase.markdown}"`);
        console.log(`   HTML: "${testCase.htmlText}"`);
        
        testCase.expectations.forEach(expectation => {
            totalTests++;
            
            try {
                const result = mapHtmlPositionToMarkdown(expectation.html, testCase.markdown);
                const passed = result === expectation.md;
                
                if (passed) {
                    passedTests++;
                    console.log(`   ✅ ${expectation.desc}: HTML ${expectation.html} -> MD ${result}`);
                } else {
                    console.log(`   ❌ ${expectation.desc}: HTML ${expectation.html} -> MD ${result} (expected ${expectation.md})`);
                }
            } catch (error) {
                console.log(`   💥 ${expectation.desc}: ERROR - ${error.message}`);
            }
        });
    });
    
    console.log(`\n🎯 Results: ${passedTests}/${totalTests} tests passed (${Math.round(passedTests/totalTests*100)}%)`);
    
    if (passedTests === totalTests) {
        console.log('🎉 All tests passed! Position mapping is working correctly.');
    } else {
        console.log('⚠️ Some tests failed. Check the implementation.');
    }
    
    return { total: totalTests, passed: passedTests };
}

// Run validation if mapHtmlPositionToMarkdown is available
if (typeof mapHtmlPositionToMarkdown === 'function') {
    validatePositionMapping();
} else {
    console.log('⏳ mapHtmlPositionToMarkdown function not found. Make sure to run this on the editor page.');
}