# OCSF Semantic Layer - PDF Documentation

This directory contains PDF exports of all web pages from the OCSF Semantic Layer project.

## Files

### Individual Pages

1. **01-dashboard.pdf** (264 KB)
   - Main interactive dashboard
   - Entity relationship graph
   - Observable analysis
   - Hot/cold path visualization

2. **02-dns-diagram.pdf** (660 KB)
   - Complete DNS semantic layer diagram
   - All 10 attributes with details
   - OCSF field mappings
   - Security metrics and relationships
   - MITRE ATT&CK coverage

3. **03-databricks-flow.pdf** (596 KB)
   - Complete data flow from raw log to business query
   - Physical vs semantic layer comparison
   - Sample data at each layer
   - Query examples

4. **04-llm-agent-usecase.pdf** (804 KB)
   - Agentic LLM workflow demonstration
   - Example conversation with AI agent
   - LLM context export details (19 KB, ~4,760 tokens)
   - MITRE ATT&CK integration
   - 27 synonyms, 12 query templates, 4 computed fields

5. **05-status.pdf** (216 KB)
   - System status page
   - File verification
   - Data previews

### Combined Document

To create a single combined PDF with all pages:

**Option 1: Using Python (PyPDF2)**
```bash
pip install PyPDF2
python3 ../combine-pdfs.py
```

**Option 2: Using Preview (macOS)**
1. Open `01-dashboard.pdf` in Preview
2. View > Thumbnails
3. Drag files 02-05 into the thumbnail sidebar
4. File > Export as PDF
5. Save as `OCSF-Semantic-Layer-Complete.pdf`

**Option 3: Using ghostscript**
```bash
brew install ghostscript
gs -dBATCH -dNOPAUSE -q -sDEVICE=pdfwrite \
   -sOutputFile=OCSF-Semantic-Layer-Complete.pdf \
   *.pdf
```

## Total Size

- Individual PDFs: ~2.5 MB
- Combined PDF: ~2.5 MB (similar, with slight compression)

## Regenerating PDFs

If you need to regenerate the PDFs (e.g., after updating content):

```bash
# Make sure web server is running
cd demo
python3 -m http.server 8888

# In another terminal, export PDFs
cd demo
./export-pdfs.sh
```

See `../PDF-EXPORT-GUIDE.md` for detailed instructions and alternative methods.

## Notes

- PDFs preserve the dark theme styling
- Mermaid diagrams are rendered as static images
- Interactive features (buttons, links) are preserved where possible
- All PDFs are print-ready with proper margins
- Generated on: January 23, 2026

## Project Information

**OCSF Semantic Layer**
- Version: 0.1.0
- OCSF Version: 1.1.0
- Repository: https://github.com/yourusername/ocsf-semantic-layer

**Key Features:**
- 27 synonym mappings for natural language queries
- 12 DNS security query templates with MITRE ATT&CK
- 4 computed fields for threat detection
- 3 MITRE techniques covered (T1071.004, T1568.002, T1048.003)
- LLM export format (19 KB / ~4,760 tokens)
- Complete Databricks implementation example
- Agentic LLM use case demonstration

## License

See project LICENSE file for details.
