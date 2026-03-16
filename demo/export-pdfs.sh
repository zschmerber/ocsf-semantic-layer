#!/bin/bash

# Export all web pages to PDF
# This script uses Chrome/Chromium in headless mode to generate PDFs

set -e

OUTPUT_DIR="demo/pdfs"
BASE_URL="http://localhost:8888"

# Create output directory
mkdir -p "$OUTPUT_DIR"

echo "🔷 OCSF Semantic Layer - PDF Export"
echo "===================================="
echo ""

# Check if Chrome is available
if command -v "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" &> /dev/null; then
    CHROME="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
elif command -v chromium &> /dev/null; then
    CHROME="chromium"
elif command -v google-chrome &> /dev/null; then
    CHROME="google-chrome"
else
    echo "❌ Error: Chrome/Chromium not found"
    echo ""
    echo "Please install Google Chrome or use one of these alternatives:"
    echo ""
    echo "Option 1: Install Chrome"
    echo "  Download from: https://www.google.com/chrome/"
    echo ""
    echo "Option 2: Use wkhtmltopdf (install via Homebrew)"
    echo "  brew install wkhtmltopdf"
    echo "  Then run: ./export-pdfs-wkhtmltopdf.sh"
    echo ""
    echo "Option 3: Print from browser"
    echo "  Open each page in your browser and use File > Print > Save as PDF"
    exit 1
fi

echo "Using: $CHROME"
echo ""

# Function to export a page to PDF
export_pdf() {
    local url=$1
    local output=$2
    local title=$3
    
    echo "📄 Exporting: $title"
    echo "   URL: $url"
    echo "   Output: $output"
    
    "$CHROME" \
        --headless \
        --disable-gpu \
        --no-sandbox \
        --print-to-pdf="$output" \
        --print-to-pdf-no-header \
        --no-pdf-header-footer \
        "$url" 2>/dev/null
    
    if [ -f "$output" ]; then
        local size=$(du -h "$output" | cut -f1)
        echo "   ✓ Success ($size)"
    else
        echo "   ✗ Failed"
    fi
    echo ""
}

# Export all pages
echo "Starting PDF export..."
echo ""

export_pdf "$BASE_URL/viz/" \
    "$OUTPUT_DIR/01-dashboard.pdf" \
    "Main Dashboard"

export_pdf "$BASE_URL/dns-diagram.html" \
    "$OUTPUT_DIR/02-dns-diagram.pdf" \
    "DNS Semantic Layer Diagram"

export_pdf "$BASE_URL/databricks-flow.html" \
    "$OUTPUT_DIR/03-databricks-flow.pdf" \
    "Databricks Data Flow"

export_pdf "$BASE_URL/llm-agent-usecase.html" \
    "$OUTPUT_DIR/04-llm-agent-usecase.pdf" \
    "Agentic LLM Use Case"

export_pdf "$BASE_URL/STATUS.html" \
    "$OUTPUT_DIR/05-status.pdf" \
    "Status Page"

echo "===================================="
echo "✅ PDF export complete!"
echo ""
echo "Output directory: $OUTPUT_DIR"
echo ""
echo "Generated files:"
ls -lh "$OUTPUT_DIR"/*.pdf 2>/dev/null || echo "No PDFs generated"
echo ""
echo "Total size:"
du -sh "$OUTPUT_DIR" 2>/dev/null || echo "0 B"
