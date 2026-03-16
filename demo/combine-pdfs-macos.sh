#!/bin/bash

# Combine PDFs using macOS built-in tools
# This uses the 'join' command from the PDFKit framework

set -e

INPUT_DIR="demo/pdfs"
OUTPUT_FILE="demo/pdfs/OCSF-Semantic-Layer-Complete.pdf"

echo "🔷 OCSF Semantic Layer - Combine PDFs (macOS)"
echo "=============================================="
echo ""

# Check if input directory exists
if [ ! -d "$INPUT_DIR" ]; then
    echo "❌ Error: Directory $INPUT_DIR not found"
    echo "Run ./export-pdfs.sh first to generate PDFs"
    exit 1
fi

# Get list of PDFs (excluding the combined one if it exists)
PDF_FILES=$(ls "$INPUT_DIR"/*.pdf 2>/dev/null | grep -v "Complete.pdf" | sort)

if [ -z "$PDF_FILES" ]; then
    echo "❌ Error: No PDF files found in $INPUT_DIR"
    exit 1
fi

echo "Found PDF files:"
for pdf in $PDF_FILES; do
    size=$(du -h "$pdf" | cut -f1)
    echo "  - $(basename "$pdf") ($size)"
done
echo ""

# Use Python with pypdf if available, otherwise suggest alternatives
if python3 -c "import pypdf" 2>/dev/null; then
    echo "Using pypdf to combine PDFs..."
    python3 << 'EOF'
from pypdf import PdfMerger
import glob
import os

merger = PdfMerger()
pdf_files = sorted([f for f in glob.glob("demo/pdfs/*.pdf") if "Complete" not in f])

for pdf in pdf_files:
    print(f"  Adding: {os.path.basename(pdf)}")
    merger.append(pdf)

output = "demo/pdfs/OCSF-Semantic-Layer-Complete.pdf"
print(f"\nWriting to: {output}")
merger.write(output)
merger.close()

size_mb = os.path.getsize(output) / 1024 / 1024
print(f"\n✅ Success! Combined PDF: {size_mb:.2f} MB")
EOF

elif python3 -c "import PyPDF2" 2>/dev/null; then
    echo "Using PyPDF2 to combine PDFs..."
    python3 << 'EOF'
from PyPDF2 import PdfMerger
import glob
import os

merger = PdfMerger()
pdf_files = sorted([f for f in glob.glob("demo/pdfs/*.pdf") if "Complete" not in f])

for pdf in pdf_files:
    print(f"  Adding: {os.path.basename(pdf)}")
    merger.append(pdf)

output = "demo/pdfs/OCSF-Semantic-Layer-Complete.pdf"
print(f"\nWriting to: {output}")
merger.write(output)
merger.close()

size_mb = os.path.getsize(output) / 1024 / 1024
print(f"\n✅ Success! Combined PDF: {size_mb:.2f} MB")
EOF

else
    echo "⚠️  Python PDF libraries not available"
    echo ""
    echo "To combine PDFs, choose one of these options:"
    echo ""
    echo "Option 1: Install pypdf"
    echo "  pip3 install pypdf"
    echo "  Then run this script again"
    echo ""
    echo "Option 2: Install PyPDF2"
    echo "  pip3 install PyPDF2"
    echo "  Then run this script again"
    echo ""
    echo "Option 3: Use Preview (macOS GUI)"
    echo "  1. Open demo/pdfs/01-dashboard.pdf in Preview"
    echo "  2. View > Thumbnails"
    echo "  3. Drag files 02-05.pdf into the thumbnail sidebar"
    echo "  4. File > Export as PDF"
    echo "  5. Save as OCSF-Semantic-Layer-Complete.pdf"
    echo ""
    echo "Option 4: Use Automator (macOS)"
    echo "  1. Open Automator"
    echo "  2. New Document > Quick Action"
    echo "  3. Add 'Combine PDF Pages' action"
    echo "  4. Select all PDFs in Finder"
    echo "  5. Right-click > Quick Actions > Combine PDF Pages"
    echo ""
    exit 1
fi
