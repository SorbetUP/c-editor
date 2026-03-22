// Test script to verify line deletion functionality
const puppeteer = require('puppeteer');

async function testLineDeletion() {
    const browser = await puppeteer.launch();
    const page = await browser.newPage();
    
    // Enable console logging
    page.on('console', (msg) => {
        console.log('PAGE LOG:', msg.text());
    });
    
    await page.goto('http://localhost:8000/docs/');
    
    // Wait for editor to load
    await page.waitForSelector('.hybrid-editor');
    
    // Test backspace on empty line
    console.log('Testing backspace on empty line...');
    
    // Click on line 6 (empty line from the logs)
    await page.click('.editor-line[data-line="6"]');
    
    // Press backspace
    await page.keyboard.press('Backspace');
    
    // Check if line was deleted/merged
    const lineCount = await page.$$eval('.editor-line', (lines) => lines.length);
    console.log(`Line count after backspace: ${lineCount}`);
    
    await browser.close();
}

if (require.main === module) {
    testLineDeletion().catch(console.error);
}