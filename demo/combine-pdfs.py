#!/usr/bin/env python3

"""
Combine all PDFs into a single document
Requires: pip install PyPDF2
"""

import os
import sys
from pathlib import Path

try:
    from PyPDF2 import PdfMerger
except ImportError:
    print("❌ Error: PyPDF2 not installed")
    print("")
    print("Install with:")
    print("  pip install PyPDF2")
    print("")
    print("Or use Preview on macOS:")
    print("  1. Open first PDF in Preview")
    print("  2. View > Thumbnails")
    print("  3. Drag other PDFs into sidebar")
    print("  4. File > Export as PDF")
    sys.exit(1)

def combine_pdfs(input_dir, output_file):
    """Combine all PDFs in input_dir into output_file"""
    
    # Get all PDF files sorted by name
    pdf_files = sorted(Path(input_dir).glob("*.pdf"))
    
    if not pdf_files:
        print(f"❌ No PDF files found in {input_dir}")
        return False
    
    print("🔷 OCSF Semantic Layer - Combine PDFs")
    print("======================================")
    print("")
    print(f"Input directory: {input_dir}")
    print(f"Output file: {output_file}")
    print("")
    print(f"Found {len(pdf_files)} PDF files:")
    for pdf in pdf_files:
        size_mb = pdf.stat().st_size / 1024 / 1024
        print(f"  - {pdf.name} ({size_mb:.2f} MB)")
    print("")
    
    # Create merger
    merger = PdfMerger()
    
    # Add each PDF
    print("Combining PDFs...")
    for pdf in pdf_files:
        print(f"  Adding: {pdf.name}")
        merger.append(str(pdf))
    
    # Write combined PDF
    print("")
    print(f"Writing combined PDF to: {output_file}")
    merger.write(output_file)
    merger.close()
    
    # Show result
    output_size = Path(output_file).stat().st_size / 1024 / 1024
    print("")
    print("✅ Success!")
    print(f"Combined PDF size: {output_size:.2f} MB")
    print(f"Location: {output_file}")
    
    return True

if __name__ == "__main__":
    input_dir = "demo/pdfs"
    output_file = "demo/pdfs/OCSF-Semantic-Layer-Complete.pdf"
    
    # Allow custom paths from command line
    if len(sys.argv) > 1:
        input_dir = sys.argv[1]
    if len(sys.argv) > 2:
        output_file = sys.argv[2]
    
    success = combine_pdfs(input_dir, output_file)
    sys.exit(0 if success else 1)
