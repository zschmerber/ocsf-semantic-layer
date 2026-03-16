# Quick Start - PDF Export

## ✅ PDFs Already Generated!

Your PDFs are ready in this directory:

```
demo/pdfs/
├── 01-dashboard.pdf          (264 KB) - Main Dashboard
├── 02-dns-diagram.pdf         (660 KB) - DNS Semantic Layer Diagram  
├── 03-databricks-flow.pdf     (596 KB) - Databricks Data Flow
├── 04-llm-agent-usecase.pdf   (804 KB) - Agentic LLM Use Case
└── 05-status.pdf              (216 KB) - Status Page

Total: 2.5 MB
```

## 📄 View PDFs

Open any PDF:
```bash
open demo/pdfs/01-dashboard.pdf
open demo/pdfs/02-dns-diagram.pdf
open demo/pdfs/03-databricks-flow.pdf
open demo/pdfs/04-llm-agent-usecase.pdf
open demo/pdfs/05-status.pdf
```

Or open all at once:
```bash
open demo/pdfs/*.pdf
```

## 🔗 Combine into Single PDF

### Option 1: Using Preview (Easiest - No Installation)

1. Open the first PDF:
   ```bash
   open demo/pdfs/01-dashboard.pdf
   ```

2. In Preview:
   - View > Thumbnails (or press `⌘⌥2`)
   - Drag files `02-05.pdf` from Finder into the thumbnail sidebar
   - File > Export as PDF
   - Save as `OCSF-Semantic-Layer-Complete.pdf`

### Option 2: Using Python (Automated)

Install pypdf:
```bash
pip3 install pypdf
```

Then run:
```bash
./demo/combine-pdfs-macos.sh
```

This creates: `demo/pdfs/OCSF-Semantic-Layer-Complete.pdf`

### Option 3: Using Automator (macOS Built-in)

1. Open Finder and navigate to `demo/pdfs/`
2. Select files `01-dashboard.pdf` through `05-status.pdf`
3. Right-click > Quick Actions > Combine PDF Pages
4. Save as `OCSF-Semantic-Layer-Complete.pdf`

## 🔄 Regenerate PDFs

If you update the web pages and need fresh PDFs:

```bash
# Make sure server is running
cd demo
python3 -m http.server 8888

# In another terminal
./demo/export-pdfs.sh
```

## 📋 What's in Each PDF

### 01-dashboard.pdf
- Interactive semantic layer dashboard
- Entity relationship graph (D3.js visualization)
- Observable analysis and hot paths
- Complete system overview

### 02-dns-diagram.pdf
- Detailed DNS semantic layer architecture
- All 10 attributes with full details
- OCSF field mappings
- 4 security metrics with formulas
- MITRE ATT&CK coverage mindmap
- Entity relationships

### 03-databricks-flow.pdf
- Complete data flow: Raw Log → Physical → Semantic → Business Query
- Side-by-side comparison of physical vs semantic queries
- Sample data at each transformation layer
- Attribute mapping table
- 3 query examples with results

### 04-llm-agent-usecase.pdf
- Agentic workflow sequence diagram
- Example conversation: Analyst → AI Agent → Databricks
- LLM context export breakdown (19 KB, ~4,760 tokens)
- 6 feature cards: entities, synonyms, MITRE, templates, computed fields, security context
- MITRE ATT&CK integration table
- Agent capabilities mindmap
- 27 synonyms, 12 query templates, 4 computed fields
- Key benefits and use cases

### 05-status.pdf
- System status and health checks
- File verification table
- DNS test event preview
- Graph data summary
- Model metadata

## 💡 Tips

- **Print Quality**: PDFs are optimized for both screen and print
- **Dark Theme**: All styling is preserved including dark backgrounds
- **Diagrams**: Mermaid diagrams are rendered as high-quality static images
- **File Size**: Total ~2.5 MB - easy to email or share
- **Links**: Internal links work within combined PDFs

## 🆘 Troubleshooting

**PDFs not opening?**
```bash
# Check they exist
ls -lh demo/pdfs/*.pdf

# Try opening with default app
open demo/pdfs/01-dashboard.pdf
```

**Need to regenerate?**
```bash
# Check server is running
lsof -i :8888

# If not, start it
cd demo && python3 -m http.server 8888

# Then export
./demo/export-pdfs.sh
```

**Combine script fails?**
```bash
# Install Python PDF library
pip3 install pypdf

# Or use Preview (GUI method above)
```

## 📚 More Information

- Full export guide: `../PDF-EXPORT-GUIDE.md`
- PDF directory README: `README.md`
- Project documentation: `../../README.md`

---

Generated: January 23, 2026
OCSF Semantic Layer v0.1.0
