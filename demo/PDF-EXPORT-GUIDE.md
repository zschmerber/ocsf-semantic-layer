# PDF Export Guide

This guide explains how to export all web pages to PDF format.

## Prerequisites

Make sure the web server is running:
```bash
cd demo
python3 -m http.server 8888
```

## Method 1: Using Chrome (Recommended)

This method uses Chrome's headless mode and works out of the box if you have Chrome installed.

```bash
cd demo
./export-pdfs.sh
```

**Requirements:**
- Google Chrome installed at `/Applications/Google Chrome.app/`

**Output:**
- PDFs will be saved to `demo/pdfs/`
- Files are numbered for easy ordering

## Method 2: Using wkhtmltopdf

This method often produces better results for complex pages with Mermaid diagrams.

**Install wkhtmltopdf:**
```bash
brew install wkhtmltopdf
```

**Run export:**
```bash
cd demo
./export-pdfs-wkhtmltopdf.sh
```

**Output:**
- PDFs will be saved to `demo/pdfs/`
- Better handling of JavaScript-rendered content

## Method 3: Manual Export (Browser)

If automated tools don't work, you can manually export each page:

1. Open each page in your browser:
   - http://localhost:8888/viz/
   - http://localhost:8888/dns-diagram.html
   - http://localhost:8888/databricks-flow.html
   - http://localhost:8888/llm-agent-usecase.html
   - http://localhost:8888/STATUS.html

2. For each page:
   - Press `Cmd+P` (or File > Print)
   - Select "Save as PDF" as the destination
   - Click "Save"

**Tips for better PDFs:**
- Wait for all diagrams to load before printing
- Use "Print backgrounds" option for better styling
- Adjust margins if content is cut off

## Method 4: Using Puppeteer (Node.js)

If you have Node.js installed, you can use Puppeteer for high-quality PDFs:

```bash
# Install Puppeteer
npm install -g puppeteer

# Create a simple export script
node demo/export-pdfs-puppeteer.js
```

## Generated Files

The export scripts create these PDFs:

1. `01-dashboard.pdf` - Main interactive dashboard
2. `02-dns-diagram.pdf` - Detailed DNS semantic layer diagram
3. `03-databricks-flow.pdf` - Databricks data flow visualization
4. `04-llm-agent-usecase.pdf` - Agentic LLM use case demonstration
5. `05-status.pdf` - System status and file checks

## Troubleshooting

### Chrome headless issues
If Chrome export fails, try:
```bash
# Check Chrome path
ls -la "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"

# Try with full path
"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" --version
```

### wkhtmltopdf issues
If wkhtmltopdf hangs or fails:
```bash
# Increase JavaScript delay
wkhtmltopdf --javascript-delay 5000 http://localhost:8888/viz/ output.pdf

# Disable JavaScript if diagrams don't render
wkhtmltopdf --disable-javascript http://localhost:8888/viz/ output.pdf
```

### Server not running
Make sure the web server is running on port 8888:
```bash
# Check if server is running
lsof -i :8888

# Start server if needed
cd demo
python3 -m http.server 8888
```

### Mermaid diagrams not rendering
Mermaid diagrams require JavaScript. Make sure:
- JavaScript is enabled in the export tool
- Sufficient delay is set for rendering (2-5 seconds)
- The page fully loads before export

## Combining PDFs

To combine all PDFs into a single document:

**Using Preview (macOS):**
1. Open first PDF in Preview
2. View > Thumbnails
3. Drag other PDFs into the thumbnail sidebar
4. File > Export as PDF

**Using command line:**
```bash
# Install ghostscript
brew install ghostscript

# Combine PDFs
gs -dBATCH -dNOPAUSE -q -sDEVICE=pdfwrite \
   -sOutputFile=demo/pdfs/OCSF-Semantic-Layer-Complete.pdf \
   demo/pdfs/*.pdf
```

## Notes

- PDF export captures the current state of the pages
- Interactive features (buttons, links) may not work in PDFs
- Mermaid diagrams are rendered as static images
- File sizes vary based on content (typically 100KB - 2MB per page)
- Dark theme styling is preserved in PDFs
