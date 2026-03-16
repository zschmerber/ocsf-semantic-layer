#!/usr/bin/env node

/**
 * Export web pages to PDF using Puppeteer
 * 
 * Install: npm install -g puppeteer
 * Run: node demo/export-pdfs-puppeteer.js
 */

const puppeteer = require('puppeteer');
const fs = require('fs');
const path = require('path');

const BASE_URL = 'http://localhost:8888';
const OUTPUT_DIR = path.join(__dirname, 'pdfs');

// Pages to export
const pages = [
  {
    url: `${BASE_URL}/viz/`,
    output: '01-dashboard.pdf',
    title: 'Main Dashboard',
    waitFor: 3000, // Wait for D3.js to render
  },
  {
    url: `${BASE_URL}/dns-diagram.html`,
    output: '02-dns-diagram.pdf',
    title: 'DNS Semantic Layer Diagram',
    waitFor: 3000, // Wait for Mermaid diagrams
  },
  {
    url: `${BASE_URL}/databricks-flow.html`,
    output: '03-databricks-flow.pdf',
    title: 'Databricks Data Flow',
    waitFor: 3000,
  },
  {
    url: `${BASE_URL}/llm-agent-usecase.html`,
    output: '04-llm-agent-usecase.pdf',
    title: 'Agentic LLM Use Case',
    waitFor: 3000,
  },
  {
    url: `${BASE_URL}/STATUS.html`,
    output: '05-status.pdf',
    title: 'Status Page',
    waitFor: 2000,
  },
];

async function exportPDF(browser, page, config) {
  console.log(`📄 Exporting: ${config.title}`);
  console.log(`   URL: ${config.url}`);
  
  try {
    await page.goto(config.url, {
      waitUntil: 'networkidle0',
      timeout: 30000,
    });
    
    // Wait for JavaScript rendering
    await page.waitForTimeout(config.waitFor);
    
    const outputPath = path.join(OUTPUT_DIR, config.output);
    
    await page.pdf({
      path: outputPath,
      format: 'Letter',
      printBackground: true,
      margin: {
        top: '10mm',
        bottom: '10mm',
        left: '10mm',
        right: '10mm',
      },
    });
    
    const stats = fs.statSync(outputPath);
    const sizeMB = (stats.size / 1024 / 1024).toFixed(2);
    console.log(`   ✓ Success (${sizeMB} MB)`);
    console.log(`   Output: ${outputPath}`);
  } catch (error) {
    console.log(`   ✗ Failed: ${error.message}`);
  }
  
  console.log('');
}

async function main() {
  console.log('🔷 OCSF Semantic Layer - PDF Export (Puppeteer)');
  console.log('=================================================');
  console.log('');
  
  // Create output directory
  if (!fs.existsSync(OUTPUT_DIR)) {
    fs.mkdirSync(OUTPUT_DIR, { recursive: true });
  }
  
  console.log('Launching browser...');
  const browser = await puppeteer.launch({
    headless: 'new',
    args: ['--no-sandbox', '--disable-setuid-sandbox'],
  });
  
  const page = await browser.newPage();
  
  // Set viewport for consistent rendering
  await page.setViewport({
    width: 1920,
    height: 1080,
    deviceScaleFactor: 2, // High DPI for better quality
  });
  
  console.log('Browser ready');
  console.log('');
  console.log('Starting PDF export...');
  console.log('');
  
  // Export each page
  for (const config of pages) {
    await exportPDF(browser, page, config);
  }
  
  await browser.close();
  
  console.log('====================================');
  console.log('✅ PDF export complete!');
  console.log('');
  console.log(`Output directory: ${OUTPUT_DIR}`);
  console.log('');
  
  // List generated files
  const files = fs.readdirSync(OUTPUT_DIR)
    .filter(f => f.endsWith('.pdf'))
    .sort();
  
  if (files.length > 0) {
    console.log('Generated files:');
    let totalSize = 0;
    files.forEach(file => {
      const filePath = path.join(OUTPUT_DIR, file);
      const stats = fs.statSync(filePath);
      const sizeMB = (stats.size / 1024 / 1024).toFixed(2);
      totalSize += stats.size;
      console.log(`  ${file} (${sizeMB} MB)`);
    });
    console.log('');
    console.log(`Total size: ${(totalSize / 1024 / 1024).toFixed(2)} MB`);
  } else {
    console.log('No PDFs generated');
  }
}

// Run
main().catch(error => {
  console.error('Error:', error);
  process.exit(1);
});
