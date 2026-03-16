#!/bin/bash

# Export all web pages to PDF using wkhtmltopdf
# Install: brew install wkhtmltopdf

set -e

OUTPUT_DIR="demo/pdfs"
BASE_URL="http://localhost:8888"

# Create output directory
mkdir -p "$OUTPUT_DIR"

echo "🔷 OCSF Semantic Layer - PDF Export (wkhtmltopdf)"
echo "=================================================="
echo ""

# Check if wkhtmltopdf is available
if ! command -v wkhtmltopdf &> /dev/null; then
    echo "❌ Error: wkhtmltopdf not found"
    echo ""
    echo "Install with Homebrew:"
    echo "  brew install wkhtmltopdf"
    echo ""
    echo "Or use the Chrome-based script:"
    echo "  ./export-pdfs.sh"
    exit 1
fi

echo "Using: $(which wkhtmltopdf)"
echo ""

# Function to export a page to PDF
export_pdf() {
    local url=$1
    local output=$2
    local title=$3
    
    echo "📄 Exporting: $title"
    echo "   URL: $url"
    echo "   Output: $output"
    
    wkhtmltopdf \
        --enable-javascript \
        --javascript-delay 2000 \
        --no-stop-slow-scripts \
        --enable-local-file-access \
        --page-size Letter \
        --margin-top 10mm \
        --margin-bottom 10mm \
        --margin-left 10mm \
        --margin-right 10mm \
        --print-media-type \
        "$url" \
        "$output" 2>/dev/null
    
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
