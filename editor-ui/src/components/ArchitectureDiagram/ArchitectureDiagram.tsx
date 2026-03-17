import { useState } from 'react';
import './ArchitectureDiagram.css';

type DiagramView = 'overview' | 'semantic' | 'comparison' | 'databricks' | 'dataflow' | 'index' | 'fullarch';

interface DiagramViewOption {
  id: DiagramView;
  label: string;
  description: string;
}

const viewOptions: DiagramViewOption[] = [
  { id: 'fullarch', label: 'Full Architecture', description: 'Complete system architecture diagram' },
  { id: 'overview', label: 'System Overview', description: 'High-level architecture of the OCSF Semantic Layer' },
  { id: 'semantic', label: 'Semantic Layer', description: 'How the semantic abstraction layer bridges physical and business concepts' },
  { id: 'comparison', label: 'Layer vs Index', description: 'How the Semantic Layer and Semantic Index differ and work together' },
  { id: 'databricks', label: 'Databricks', description: 'Deployment flow for Databricks with Unity Catalog and Delta Lake' },
  { id: 'dataflow', label: 'Data Flow', description: 'How data flows through the semantic layer' },
  { id: 'index', label: 'Index Structure', description: 'Semantic index components and relationships' },
];

/**
 * Architecture diagram component showing system design.
 * Provides visual documentation of the OCSF Semantic Layer architecture.
 */
export function ArchitectureDiagram() {
  const [activeView, setActiveView] = useState<DiagramView>('overview');

  return (
    <div className="architecture-diagram">
      <div className="architecture-header">
        <h2>🏗️ Architecture</h2>
        <p className="architecture-subtitle">
          Visual documentation of the OCSF Semantic Layer system design
        </p>
      </div>

      <div className="architecture-view-selector">
        {viewOptions.map((option) => (
          <button
            key={option.id}
            className={`view-option ${activeView === option.id ? 'active' : ''}`}
            onClick={() => setActiveView(option.id)}
          >
            <span className="view-option-label">{option.label}</span>
            <span className="view-option-desc">{option.description}</span>
          </button>
        ))}
      </div>

      <div className="architecture-content">
        {activeView === 'fullarch' && <FullArchitectureDiagram />}
        {activeView === 'overview' && <OverviewDiagram />}
        {activeView === 'semantic' && <SemanticLayerDiagram />}
        {activeView === 'comparison' && <LayerVsIndexDiagram />}
        {activeView === 'databricks' && <DatabricksDiagram />}
        {activeView === 'dataflow' && <DataFlowDiagram />}
        {activeView === 'index' && <IndexStructureDiagram />}
      </div>
    </div>
  );
}

function FullArchitectureDiagram() {
  // Architecture diagram showing the full system
  // Image should be placed at editor-ui/public/architecture-diagram.png
  return (
    <div className="diagram-container full-arch">
      <div className="architecture-image-container">
        <img 
          src="/architecture-diagram.png" 
          alt="OCSF Semantic Layer Full Architecture"
          className="architecture-image"
          onError={(e) => {
            // If image not found, show placeholder message
            const target = e.target as HTMLImageElement;
            target.style.display = 'none';
            const placeholder = target.nextElementSibling as HTMLElement;
            if (placeholder) placeholder.style.display = 'flex';
          }}
        />
        <div className="architecture-image-placeholder" style={{ display: 'none' }}>
          <div className="placeholder-content">
            <span className="placeholder-icon">🖼️</span>
            <p>Architecture diagram not found</p>
            <p className="placeholder-hint">
              Place the architecture image at:<br/>
              <code>editor-ui/public/architecture-diagram.png</code>
            </p>
          </div>
        </div>
      </div>
      <div className="diagram-legend">
        <h4>Architecture Components</h4>
        <ul>
          <li><strong>API Layer:</strong> FastAPI server with Query and Index Management endpoints</li>
          <li><strong>Query Layer:</strong> Query Engine and Optimizer with Redis + S3 caching</li>
          <li><strong>Data Ingestion:</strong> ETL Pipeline transforming raw logs (Zeek, Windows, Proxy) to OCSF</li>
          <li><strong>Physical Layer:</strong> Iceberg tables storing OCSF-normalized data (network_activity, authentication, file_activity)</li>
          <li><strong>Semantic Layer:</strong> PostgreSQL Index with Table Metadata, Field Lineage, Partition Index, and Version Mappings</li>
        </ul>
      </div>
    </div>
  );
}

function SemanticLayerDiagram() {
  return (
    <div className="diagram-container semantic-layer-diagram">
      <svg viewBox="0 0 960 720" className="architecture-svg">
        <defs>
          <marker id="slArrow" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
            <polygon points="0 0, 10 3.5, 0 7" fill="#667eea"/>
          </marker>
          <marker id="slArrowGreen" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
            <polygon points="0 0, 10 3.5, 0 7" fill="#48bb78"/>
          </marker>
          <marker id="slArrowOrange" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
            <polygon points="0 0, 10 3.5, 0 7" fill="#ed8936"/>
          </marker>
          <marker id="slArrowPurple" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
            <polygon points="0 0, 10 3.5, 0 7" fill="#9f7aea"/>
          </marker>
          <linearGradient id="semanticGrad" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor="#667eea" stopOpacity="0.15"/>
            <stop offset="100%" stopColor="#764ba2" stopOpacity="0.08"/>
          </linearGradient>
          <filter id="slShadow" x="-5%" y="-5%" width="110%" height="110%">
            <feDropShadow dx="1" dy="2" stdDeviation="2" floodOpacity="0.15"/>
          </filter>
        </defs>

        {/* Title */}
        <text x="480" y="30" textAnchor="middle" className="diagram-title">
          OCSF Semantic Layer — Bridging Physical Schema &amp; Business Concepts
        </text>

        {/* ── Physical Layer (bottom) ── */}
        <g transform="translate(30, 580)">
          <rect x="0" y="0" width="900" height="120" rx="10" fill="#f3e5f5" stroke="#ce93d8" strokeWidth="1.5" filter="url(#slShadow)"/>
          <text x="450" y="24" textAnchor="middle" className="sl-layer-title" fill="#7b1fa2">Physical Layer — OCSF Schema</text>

          <rect x="30" y="40" width="120" height="55" rx="6" className="sl-physical-box"/>
          <text x="90" y="62" textAnchor="middle" className="sl-box-label">Event Classes</text>
          <text x="90" y="78" textAnchor="middle" className="sl-box-sub">4001, 3001, 1001…</text>

          <rect x="175" y="40" width="120" height="55" rx="6" className="sl-physical-box"/>
          <text x="235" y="62" textAnchor="middle" className="sl-box-label">Objects</text>
          <text x="235" y="78" textAnchor="middle" className="sl-box-sub">actor, device, user</text>

          <rect x="320" y="40" width="120" height="55" rx="6" className="sl-physical-box"/>
          <text x="380" y="62" textAnchor="middle" className="sl-box-label">Attributes</text>
          <text x="380" y="78" textAnchor="middle" className="sl-box-sub">type_uid, severity</text>

          <rect x="465" y="40" width="120" height="55" rx="6" className="sl-physical-box"/>
          <text x="525" y="62" textAnchor="middle" className="sl-box-label">Observables</text>
          <text x="525" y="78" textAnchor="middle" className="sl-box-sub">IP, domain, hash</text>

          <rect x="610" y="40" width="120" height="55" rx="6" className="sl-physical-box"/>
          <text x="670" y="62" textAnchor="middle" className="sl-box-label">Categories</text>
          <text x="670" y="78" textAnchor="middle" className="sl-box-sub">IAM, Network, System</text>

          <rect x="755" y="40" width="120" height="55" rx="6" className="sl-physical-box"/>
          <text x="815" y="62" textAnchor="middle" className="sl-box-label">Dictionary</text>
          <text x="815" y="78" textAnchor="middle" className="sl-box-sub">Field definitions</text>
        </g>

        {/* ── Semantic Abstraction Layer (center) ── */}
        <g transform="translate(30, 280)">
          <rect x="0" y="0" width="900" height="270" rx="10" fill="url(#semanticGrad)" stroke="#667eea" strokeWidth="2" filter="url(#slShadow)"/>
          <text x="450" y="24" textAnchor="middle" className="sl-layer-title" fill="#4c51bf">Semantic Abstraction Layer</text>

          {/* Entities */}
          <g transform="translate(25, 40)">
            <rect x="0" y="0" width="160" height="90" rx="6" className="sl-semantic-box entity"/>
            <text x="80" y="22" textAnchor="middle" className="sl-box-title">Entities</text>
            <text x="80" y="42" textAnchor="middle" className="sl-box-sub">Business objects mapped</text>
            <text x="80" y="57" textAnchor="middle" className="sl-box-sub">to OCSF event classes</text>
            <text x="80" y="78" textAnchor="middle" className="sl-box-example">e.g. "Login Event"</text>
          </g>

          {/* Metrics */}
          <g transform="translate(210, 40)">
            <rect x="0" y="0" width="160" height="90" rx="6" className="sl-semantic-box metric"/>
            <text x="80" y="22" textAnchor="middle" className="sl-box-title">Metrics</text>
            <text x="80" y="42" textAnchor="middle" className="sl-box-sub">Computed measures</text>
            <text x="80" y="57" textAnchor="middle" className="sl-box-sub">with SQL expressions</text>
            <text x="80" y="78" textAnchor="middle" className="sl-box-example">e.g. "Failed Login Rate"</text>
          </g>

          {/* Datasets */}
          <g transform="translate(395, 40)">
            <rect x="0" y="0" width="160" height="90" rx="6" className="sl-semantic-box dataset"/>
            <text x="80" y="22" textAnchor="middle" className="sl-box-title">Datasets</text>
            <text x="80" y="42" textAnchor="middle" className="sl-box-sub">Curated collections</text>
            <text x="80" y="57" textAnchor="middle" className="sl-box-sub">of entities &amp; metrics</text>
            <text x="80" y="78" textAnchor="middle" className="sl-box-example">e.g. "Auth Analytics"</text>
          </g>

          {/* Relationships */}
          <g transform="translate(580, 40)">
            <rect x="0" y="0" width="160" height="90" rx="6" className="sl-semantic-box relationship"/>
            <text x="80" y="22" textAnchor="middle" className="sl-box-title">Relationships</text>
            <text x="80" y="42" textAnchor="middle" className="sl-box-sub">Entity connections</text>
            <text x="80" y="57" textAnchor="middle" className="sl-box-sub">with role aliases</text>
            <text x="80" y="78" textAnchor="middle" className="sl-box-example">e.g. "user → device"</text>
          </g>

          {/* Hierarchy */}
          <g transform="translate(765, 40)">
            <rect x="0" y="0" width="110" height="90" rx="6" className="sl-semantic-box hierarchy"/>
            <text x="55" y="22" textAnchor="middle" className="sl-box-title">Hierarchy</text>
            <text x="55" y="42" textAnchor="middle" className="sl-box-sub">Folder groups</text>
            <text x="55" y="57" textAnchor="middle" className="sl-box-sub">&amp; ordering</text>
            <text x="55" y="78" textAnchor="middle" className="sl-box-example">IAM / Network</text>
          </g>

          {/* Validation & Schema Gen row */}
          <g transform="translate(25, 155)">
            <rect x="0" y="0" width="250" height="55" rx="6" className="sl-engine-box"/>
            <text x="125" y="22" textAnchor="middle" className="sl-box-title">Validation Engine</text>
            <text x="125" y="42" textAnchor="middle" className="sl-box-sub">Schema conformance &amp; constraint checks</text>
          </g>

          <g transform="translate(300, 155)">
            <rect x="0" y="0" width="250" height="55" rx="6" className="sl-engine-box"/>
            <text x="125" y="22" textAnchor="middle" className="sl-box-title">Schema Generator</text>
            <text x="125" y="42" textAnchor="middle" className="sl-box-sub">YAML ↔ Rust model serialization</text>
          </g>

          <g transform="translate(575, 155)">
            <rect x="0" y="0" width="300" height="55" rx="6" className="sl-engine-box"/>
            <text x="150" y="22" textAnchor="middle" className="sl-box-title">Vector Embeddings &amp; Mapping Assistant</text>
            <text x="150" y="42" textAnchor="middle" className="sl-box-sub">AI-assisted field mapping via similarity search</text>
          </g>

          {/* Internal arrows: entities → metrics, entities → relationships */}
          <line x1="185" y1="85" x2="210" y2="85" stroke="#667eea" strokeWidth="1.5" markerEnd="url(#slArrow)"/>
          <line x1="370" y1="85" x2="395" y2="85" stroke="#667eea" strokeWidth="1.5" markerEnd="url(#slArrow)"/>
          <line x1="555" y1="85" x2="580" y2="85" stroke="#667eea" strokeWidth="1.5" markerEnd="url(#slArrow)"/>
        </g>

        {/* ── Business / Consumer Layer (top) ── */}
        <g transform="translate(30, 50)">
          <rect x="0" y="0" width="900" height="200" rx="10" fill="#e8f5e9" stroke="#66bb6a" strokeWidth="1.5" filter="url(#slShadow)"/>
          <text x="450" y="24" textAnchor="middle" className="sl-layer-title" fill="#2e7d32">Business &amp; Consumer Layer</text>

          {/* Query Translation */}
          <g transform="translate(25, 40)">
            <rect x="0" y="0" width="200" height="70" rx="6" className="sl-consumer-box"/>
            <text x="100" y="22" textAnchor="middle" className="sl-box-title">Query Translation</text>
            <text x="100" y="42" textAnchor="middle" className="sl-box-sub">Business terms →</text>
            <text x="100" y="57" textAnchor="middle" className="sl-box-sub">OCSF-based SQL</text>
          </g>

          {/* Warehouse Artifacts */}
          <g transform="translate(250, 40)">
            <rect x="0" y="0" width="200" height="70" rx="6" className="sl-consumer-box"/>
            <text x="100" y="22" textAnchor="middle" className="sl-box-title">Warehouse Artifacts</text>
            <text x="100" y="42" textAnchor="middle" className="sl-box-sub">dbt models, Cube.js,</text>
            <text x="100" y="57" textAnchor="middle" className="sl-box-sub">SQL views, ETL configs</text>
          </g>

          {/* Detection Coverage */}
          <g transform="translate(475, 40)">
            <rect x="0" y="0" width="200" height="70" rx="6" className="sl-consumer-box"/>
            <text x="100" y="22" textAnchor="middle" className="sl-box-title">Detection Coverage</text>
            <text x="100" y="42" textAnchor="middle" className="sl-box-sub">MITRE ATT&amp;CK mapping</text>
            <text x="100" y="57" textAnchor="middle" className="sl-box-sub">&amp; gap analysis</text>
          </g>

          {/* Catalog & Plugins */}
          <g transform="translate(700, 40)">
            <rect x="0" y="0" width="175" height="70" rx="6" className="sl-consumer-box"/>
            <text x="87" y="22" textAnchor="middle" className="sl-box-title">Catalog Plugins</text>
            <text x="87" y="42" textAnchor="middle" className="sl-box-sub">Iceberg, Delta Lake</text>
            <text x="87" y="57" textAnchor="middle" className="sl-box-sub">sidecar metadata</text>
          </g>

          {/* User personas */}
          <g transform="translate(25, 130)">
            <rect x="0" y="0" width="850" height="50" rx="6" fill="white" fillOpacity="0.7" stroke="#a5d6a7" strokeWidth="1"/>
            <text x="30" y="30" className="sl-persona">👤 Security Analyst</text>
            <text x="240" y="30" className="sl-persona">🔧 Data Engineer</text>
            <text x="440" y="30" className="sl-persona">🛡️ Threat Intel Team</text>
            <text x="650" y="30" className="sl-persona">🏗️ Data Architect</text>
          </g>
        </g>

        {/* ── Vertical flow arrows between layers ── */}
        {/* Business → Semantic */}
        <line x1="155" y1="250" x2="155" y2="280" stroke="#48bb78" strokeWidth="2" markerEnd="url(#slArrowGreen)"/>
        <line x1="380" y1="250" x2="380" y2="280" stroke="#48bb78" strokeWidth="2" markerEnd="url(#slArrowGreen)"/>
        <line x1="605" y1="250" x2="605" y2="280" stroke="#48bb78" strokeWidth="2" markerEnd="url(#slArrowGreen)"/>
        <line x1="817" y1="250" x2="817" y2="280" stroke="#48bb78" strokeWidth="2" markerEnd="url(#slArrowGreen)"/>

        {/* Semantic → Physical */}
        <line x1="155" y1="550" x2="155" y2="580" stroke="#9f7aea" strokeWidth="2" markerEnd="url(#slArrowPurple)"/>
        <line x1="380" y1="550" x2="380" y2="580" stroke="#9f7aea" strokeWidth="2" markerEnd="url(#slArrowPurple)"/>
        <line x1="605" y1="550" x2="605" y2="580" stroke="#9f7aea" strokeWidth="2" markerEnd="url(#slArrowPurple)"/>
        <line x1="817" y1="550" x2="817" y2="580" stroke="#9f7aea" strokeWidth="2" markerEnd="url(#slArrowPurple)"/>

        {/* Bidirectional labels */}
        <text x="20" y="268" className="sl-arrow-label" fill="#48bb78">consumes</text>
        <text x="20" y="568" className="sl-arrow-label" fill="#9f7aea">maps to</text>
      </svg>

      <div className="diagram-legend">
        <h4>How the Semantic Layer Works</h4>
        <ul>
          <li><strong>Physical Layer:</strong> The raw OCSF schema — event classes (Authentication, DNS Activity, etc.), objects, attributes, observables, and the data dictionary.</li>
          <li><strong>Semantic Abstraction:</strong> Maps physical schema elements to business-friendly concepts. Entities wrap event classes with descriptions and field selections. Metrics define computed measures (e.g., "failed login rate"). Datasets group entities and metrics into curated analytical collections. Relationships connect entities with role aliases.</li>
          <li><strong>Validation &amp; Generation:</strong> The validation engine checks that semantic definitions conform to the underlying OCSF schema. The schema generator serializes models to/from YAML. The vector embedding engine powers AI-assisted field mapping suggestions.</li>
          <li><strong>Business Layer:</strong> Consumers query using business terms that get translated to OCSF SQL. Warehouse artifacts (dbt, Cube.js) are auto-generated. Detection coverage maps entities to MITRE ATT&amp;CK. Catalog plugins (Iceberg, Delta) sync metadata as sidecars.</li>
        </ul>
      </div>
    </div>
  );
}

function LayerVsIndexDiagram() {
  return (
    <div className="diagram-container layer-vs-index">
      <svg viewBox="0 0 920 520" className="architecture-svg">
        <defs>
          <marker id="cmpArrow" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
            <polygon points="0 0, 10 3.5, 0 7" fill="#667eea"/>
          </marker>
          <marker id="cmpArrowTeal" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
            <polygon points="0 0, 10 3.5, 0 7" fill="#38b2ac"/>
          </marker>
          <filter id="cmpShadow" x="-5%" y="-5%" width="110%" height="110%">
            <feDropShadow dx="1" dy="2" stdDeviation="2" floodOpacity="0.12"/>
          </filter>
        </defs>

        <text x="460" y="30" textAnchor="middle" className="diagram-title">
          Semantic Layer vs Semantic Index
        </text>

        {/* ── Left: Semantic Layer ── */}
        <g transform="translate(20, 55)">
          <rect x="0" y="0" width="410" height="400" rx="10" fill="rgba(102,126,234,0.08)" stroke="#667eea" strokeWidth="2" filter="url(#cmpShadow)"/>
          <text x="205" y="28" textAnchor="middle" className="cmp-section-title" fill="#4c51bf">Semantic Layer (ocsf-semantic)</text>
          <text x="205" y="48" textAnchor="middle" className="cmp-subtitle">"What does this data mean?"</text>

          {/* Boxes */}
          <g transform="translate(20, 70)">
            <rect x="0" y="0" width="170" height="65" rx="6" fill="white" stroke="#4299e1" strokeWidth="1.5"/>
            <text x="85" y="20" textAnchor="middle" className="cmp-box-title">Entities</text>
            <text x="85" y="38" textAnchor="middle" className="cmp-box-desc">Business objects mapped</text>
            <text x="85" y="52" textAnchor="middle" className="cmp-box-desc">to OCSF event classes</text>
          </g>
          <g transform="translate(220, 70)">
            <rect x="0" y="0" width="170" height="65" rx="6" fill="white" stroke="#ed8936" strokeWidth="1.5"/>
            <text x="85" y="20" textAnchor="middle" className="cmp-box-title">Metrics</text>
            <text x="85" y="38" textAnchor="middle" className="cmp-box-desc">Computed measures with</text>
            <text x="85" y="52" textAnchor="middle" className="cmp-box-desc">SQL expressions</text>
          </g>
          <g transform="translate(20, 150)">
            <rect x="0" y="0" width="170" height="65" rx="6" fill="white" stroke="#48bb78" strokeWidth="1.5"/>
            <text x="85" y="20" textAnchor="middle" className="cmp-box-title">Datasets</text>
            <text x="85" y="38" textAnchor="middle" className="cmp-box-desc">Curated collections of</text>
            <text x="85" y="52" textAnchor="middle" className="cmp-box-desc">entities &amp; metrics</text>
          </g>
          <g transform="translate(220, 150)">
            <rect x="0" y="0" width="170" height="65" rx="6" fill="white" stroke="#9f7aea" strokeWidth="1.5"/>
            <text x="85" y="20" textAnchor="middle" className="cmp-box-title">Relationships</text>
            <text x="85" y="38" textAnchor="middle" className="cmp-box-desc">Entity connections</text>
            <text x="85" y="52" textAnchor="middle" className="cmp-box-desc">with role aliases</text>
          </g>

          {/* Example */}
          <g transform="translate(20, 240)">
            <rect x="0" y="0" width="370" height="70" rx="6" fill="#f7fafc" stroke="#e2e8f0" strokeWidth="1"/>
            <text x="185" y="18" textAnchor="middle" className="cmp-example-title">Example</text>
            <text x="185" y="36" textAnchor="middle" className="cmp-example">"Login Event" = OCSF class 3002</text>
            <text x="185" y="52" textAnchor="middle" className="cmp-example">"Failed Login Rate" = COUNT(*) WHERE status = 'failure'</text>
          </g>

          {/* Role label */}
          <g transform="translate(20, 330)">
            <rect x="0" y="0" width="370" height="50" rx="6" fill="#667eea" fillOpacity="0.1" stroke="#667eea" strokeWidth="1" strokeDasharray="4"/>
            <text x="185" y="20" textAnchor="middle" className="cmp-role-label">Design-time modeling &amp; authoring</text>
            <text x="185" y="38" textAnchor="middle" className="cmp-role-sub">Defines meaning, validates against OCSF schema</text>
          </g>
        </g>

        {/* ── Right: Semantic Index ── */}
        <g transform="translate(490, 55)">
          <rect x="0" y="0" width="410" height="400" rx="10" fill="rgba(56,178,172,0.08)" stroke="#38b2ac" strokeWidth="2" filter="url(#cmpShadow)"/>
          <text x="205" y="28" textAnchor="middle" className="cmp-section-title" fill="#285e61">Semantic Index (ocsf-index)</text>
          <text x="205" y="48" textAnchor="middle" className="cmp-subtitle">"Where is the data &amp; how to get it fast?"</text>

          {/* Boxes */}
          <g transform="translate(20, 70)">
            <rect x="0" y="0" width="170" height="65" rx="6" fill="white" stroke="#38b2ac" strokeWidth="1.5"/>
            <text x="85" y="20" textAnchor="middle" className="cmp-box-title">Table Registry</text>
            <text x="85" y="38" textAnchor="middle" className="cmp-box-desc">Physical table locations</text>
            <text x="85" y="52" textAnchor="middle" className="cmp-box-desc">&amp; class_uid mapping</text>
          </g>
          <g transform="translate(220, 70)">
            <rect x="0" y="0" width="170" height="65" rx="6" fill="white" stroke="#38b2ac" strokeWidth="1.5"/>
            <text x="85" y="20" textAnchor="middle" className="cmp-box-title">Lineage Tracking</text>
            <text x="85" y="38" textAnchor="middle" className="cmp-box-desc">Source → OCSF field</text>
            <text x="85" y="52" textAnchor="middle" className="cmp-box-desc">maps &amp; transformations</text>
          </g>
          <g transform="translate(20, 150)">
            <rect x="0" y="0" width="170" height="65" rx="6" fill="white" stroke="#38b2ac" strokeWidth="1.5"/>
            <text x="85" y="20" textAnchor="middle" className="cmp-box-title">Partitions &amp; Stats</text>
            <text x="85" y="38" textAnchor="middle" className="cmp-box-desc">Time bounds, row counts,</text>
            <text x="85" y="52" textAnchor="middle" className="cmp-box-desc">cardinality, null ratios</text>
          </g>
          <g transform="translate(220, 150)">
            <rect x="0" y="0" width="170" height="65" rx="6" fill="white" stroke="#38b2ac" strokeWidth="1.5"/>
            <text x="85" y="20" textAnchor="middle" className="cmp-box-title">Query Cache</text>
            <text x="85" y="38" textAnchor="middle" className="cmp-box-desc">TTL-based caching</text>
            <text x="85" y="52" textAnchor="middle" className="cmp-box-desc">with LRU eviction</text>
          </g>

          {/* Example */}
          <g transform="translate(20, 240)">
            <rect x="0" y="0" width="370" height="70" rx="6" fill="#f0fff4" stroke="#c6f6d5" strokeWidth="1"/>
            <text x="185" y="18" textAnchor="middle" className="cmp-example-title">Example</text>
            <text x="185" y="36" textAnchor="middle" className="cmp-example">auth table: 2.4M rows, partitioned to 2026-03-15</text>
            <text x="185" y="52" textAnchor="middle" className="cmp-example">user_name → actor.user.name (confidence: 0.95)</text>
          </g>

          {/* Role label */}
          <g transform="translate(20, 330)">
            <rect x="0" y="0" width="370" height="50" rx="6" fill="#38b2ac" fillOpacity="0.1" stroke="#38b2ac" strokeWidth="1" strokeDasharray="4"/>
            <text x="185" y="20" textAnchor="middle" className="cmp-role-label">Runtime metadata &amp; query optimization</text>
            <text x="185" y="38" textAnchor="middle" className="cmp-role-sub">Tracks physical state, enables fast queries</text>
          </g>
        </g>

        {/* ── Center connecting arrow ── */}
        <line x1="430" y1="200" x2="490" y2="200" stroke="#667eea" strokeWidth="2" markerEnd="url(#cmpArrow)"/>
        <line x1="490" y1="220" x2="430" y2="220" stroke="#38b2ac" strokeWidth="2" markerEnd="url(#cmpArrowTeal)"/>
        <text x="460" y="185" textAnchor="middle" className="cmp-arrow-label">registers</text>
        <text x="460" y="245" textAnchor="middle" className="cmp-arrow-label">optimizes</text>

        {/* ── Bottom: Query flow ── */}
        <g transform="translate(20, 475)">
          <rect x="0" y="0" width="880" height="35" rx="6" fill="#f7fafc" stroke="#e2e8f0" strokeWidth="1"/>
          <text x="440" y="14" textAnchor="middle" className="cmp-flow-title">Query: "Show me failed logins this week"</text>
          <text x="440" y="28" textAnchor="middle" className="cmp-flow-desc">Layer resolves meaning (class 3002, status=failure) → Index finds partitions &amp; plans scan → Results returned</text>
        </g>
      </svg>

      <div className="diagram-legend">
        <h4>Semantic Layer vs Semantic Index</h4>
        <ul>
          <li><strong>Semantic Layer (ocsf-semantic):</strong> The modeling surface. Defines business-friendly abstractions — entities that wrap OCSF event classes, metrics with SQL formulas, datasets grouping related concepts, and relationships between entities. This is where you author "what does this data mean in business terms." It validates definitions against the OCSF schema and generates YAML models.</li>
          <li><strong>Semantic Index (ocsf-index):</strong> The operational metadata engine. Tracks the physical reality of your data at runtime — which tables exist and where, how raw source fields map to OCSF fields (with confidence scores), partition time bounds for query pruning, column statistics for query planning, and a TTL-based query cache. This is "where is the data and how do I efficiently get to it."</li>
          <li><strong>How they work together:</strong> When a user queries "show me failed logins," the Semantic Layer resolves what that means in OCSF terms (event class 3002, status_id = 2). The Semantic Index then determines which physical partitions to scan, uses column statistics for the query plan, checks the cache for recent results, and returns the data. The Layer defines meaning; the Index makes it fast.</li>
        </ul>
      </div>
    </div>
  );
}

function DatabricksDiagram() {
  return (
    <div className="diagram-container databricks-diagram">
      <svg viewBox="0 0 960 1080" className="architecture-svg">
        <defs>
          <marker id="dbArrow" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
            <polygon points="0 0, 10 3.5, 0 7" fill="#ff3621"/>
          </marker>
          <marker id="dbArrowBlue" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
            <polygon points="0 0, 10 3.5, 0 7" fill="#667eea"/>
          </marker>
          <marker id="dbArrowTeal" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
            <polygon points="0 0, 10 3.5, 0 7" fill="#38b2ac"/>
          </marker>
          <marker id="dbArrowGreen" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
            <polygon points="0 0, 10 3.5, 0 7" fill="#48bb78"/>
          </marker>
          <filter id="dbShadow" x="-5%" y="-5%" width="110%" height="110%">
            <feDropShadow dx="1" dy="2" stdDeviation="2" floodOpacity="0.12"/>
          </filter>
        </defs>

        <text x="480" y="28" textAnchor="middle" className="diagram-title">
          Databricks Deployment — External ETL + Cost-Optimized Architecture
        </text>

        {/* ── Raw Data Sources (top-left) ── */}
        <g transform="translate(20, 50)">
          <rect x="0" y="0" width="200" height="130" rx="8" fill="#fff3e0" stroke="#ffb74d" strokeWidth="1.5" filter="url(#dbShadow)"/>
          <text x="100" y="22" textAnchor="middle" className="db-section-title" fill="#e65100">Raw Log Sources</text>
          <rect x="15" y="35" width="170" height="25" rx="4" fill="white" stroke="#ffe0b2" strokeWidth="1"/>
          <text x="100" y="52" textAnchor="middle" className="db-item">S3: Zeek / Suricata logs</text>
          <rect x="15" y="68" width="170" height="25" rx="4" fill="white" stroke="#ffe0b2" strokeWidth="1"/>
          <text x="100" y="85" textAnchor="middle" className="db-item">Kafka: CloudTrail, VPC Flow</text>
          <rect x="15" y="101" width="170" height="22" rx="4" fill="white" stroke="#ffe0b2" strokeWidth="1"/>
          <text x="100" y="116" textAnchor="middle" className="db-item-dim">Syslog, Beats, custom feeds</text>
        </g>

        {/* Arrow: Sources → External ETL */}
        <line x1="120" y1="180" x2="120" y2="215" stroke="#e65100" strokeWidth="2" markerEnd="url(#dbArrow)"/>
        <text x="155" y="202" className="db-arrow-label" fill="#e65100">ingest</text>

        {/* ── External Rust ETL Service (container) ── */}
        <g transform="translate(20, 220)">
          <rect x="0" y="0" width="440" height="130" rx="8" fill="#e3f2fd" stroke="#1565c0" strokeWidth="2" filter="url(#dbShadow)"/>
          <text x="220" y="22" textAnchor="middle" className="db-section-title" fill="#1565c0">External ETL — Rust Container Service</text>
          <text x="220" y="40" textAnchor="middle" className="db-item-dim">Runs on ECS / EKS / bare metal — outside Databricks compute</text>

          <rect x="15" y="50" width="125" height="65" rx="5" fill="white" stroke="#90caf9" strokeWidth="1"/>
          <text x="77" y="68" textAnchor="middle" className="db-box-title">ocsf-core</text>
          <text x="77" y="82" textAnchor="middle" className="db-item-dim">OCSF normalization</text>
          <text x="77" y="96" textAnchor="middle" className="db-item-dim">Observable extraction</text>
          <text x="77" y="108" textAnchor="middle" className="db-item-dim">Schema validation</text>

          <rect x="155" y="50" width="130" height="65" rx="5" fill="white" stroke="#90caf9" strokeWidth="1"/>
          <text x="220" y="68" textAnchor="middle" className="db-box-title">ocsf-warehouse</text>
          <text x="220" y="82" textAnchor="middle" className="db-item-dim">Delta Lake writer</text>
          <text x="220" y="96" textAnchor="middle" className="db-item-dim">Parquet partitioning</text>
          <text x="220" y="108" textAnchor="middle" className="db-item-dim">ZORDER prep</text>

          <rect x="300" y="50" width="125" height="65" rx="5" fill="white" stroke="#90caf9" strokeWidth="1"/>
          <text x="362" y="68" textAnchor="middle" className="db-box-title">ocsf-catalog</text>
          <text x="362" y="82" textAnchor="middle" className="db-item-dim">Delta / Iceberg plugin</text>
          <text x="362" y="96" textAnchor="middle" className="db-item-dim">Table registration</text>
          <text x="362" y="108" textAnchor="middle" className="db-item-dim">Sidecar metadata</text>
        </g>

        {/* ── Semantic Index — PostgreSQL (external) ── */}
        <g transform="translate(500, 220)">
          <rect x="0" y="0" width="220" height="130" rx="8" fill="rgba(56,178,172,0.08)" stroke="#38b2ac" strokeWidth="2" filter="url(#dbShadow)"/>
          <text x="110" y="22" textAnchor="middle" className="db-section-title" fill="#285e61">Semantic Index</text>
          <text x="110" y="38" textAnchor="middle" className="db-item-dim">PostgreSQL (external)</text>

          <rect x="15" y="48" width="90" height="32" rx="4" fill="white" stroke="#38b2ac" strokeWidth="1"/>
          <text x="60" y="68" textAnchor="middle" className="db-item">table_registry</text>
          <rect x="115" y="48" width="90" height="32" rx="4" fill="white" stroke="#38b2ac" strokeWidth="1"/>
          <text x="160" y="68" textAnchor="middle" className="db-item">source_lineage</text>
          <rect x="15" y="88" width="90" height="32" rx="4" fill="white" stroke="#38b2ac" strokeWidth="1"/>
          <text x="60" y="108" textAnchor="middle" className="db-item">field_lineage</text>
          <rect x="115" y="88" width="90" height="32" rx="4" fill="white" stroke="#38b2ac" strokeWidth="1"/>
          <text x="160" y="108" textAnchor="middle" className="db-item">partition_meta</text>
        </g>

        {/* Arrow: ETL → Semantic Index */}
        <path d="M 460 285 L 500 285" fill="none" stroke="#38b2ac" strokeWidth="2" markerEnd="url(#dbArrowTeal)"/>
        <text x="480" y="278" textAnchor="middle" className="db-arrow-label" fill="#38b2ac">registers</text>

        {/* ── OCSF Semantic Editor (right side) ── */}
        <g transform="translate(750, 220)">
          <rect x="0" y="0" width="190" height="130" rx="8" fill="rgba(102,126,234,0.1)" stroke="#667eea" strokeWidth="2" filter="url(#dbShadow)"/>
          <text x="95" y="22" textAnchor="middle" className="db-section-title" fill="#4c51bf">OCSF Semantic</text>
          <text x="95" y="38" textAnchor="middle" className="db-section-title" fill="#4c51bf">Editor</text>
          <rect x="15" y="48" width="160" height="22" rx="4" fill="white" stroke="#c3dafe" strokeWidth="1"/>
          <text x="95" y="63" textAnchor="middle" className="db-item">semantic-model.yaml</text>
          <rect x="15" y="78" width="160" height="22" rx="4" fill="white" stroke="#c3dafe" strokeWidth="1"/>
          <text x="95" y="93" textAnchor="middle" className="db-item">SQL views, dbt, Cube.js</text>
          <rect x="15" y="108" width="160" height="16" rx="4" fill="#ede7f6" stroke="#b39ddb" strokeWidth="1"/>
          <text x="95" y="120" textAnchor="middle" className="db-item-dim">Generates artifacts</text>
        </g>

        {/* Arrow: ETL → S3 */}
        <line x1="240" y1="350" x2="240" y2="385" stroke="#1565c0" strokeWidth="2" markerEnd="url(#dbArrowBlue)"/>
        <text x="275" y="372" className="db-arrow-label" fill="#1565c0">writes Delta</text>


        {/* ── S3 Bucket — Delta Lake Storage ── */}
        <g transform="translate(20, 390)">
          <rect x="0" y="0" width="700" height="140" rx="8" fill="#e8f5e9" stroke="#66bb6a" strokeWidth="2" filter="url(#dbShadow)"/>
          <text x="350" y="22" textAnchor="middle" className="db-section-title" fill="#2e7d32">S3 Bucket — Delta Lake Tables (Physical Storage)</text>
          <text x="350" y="38" textAnchor="middle" className="db-item-dim">s3://ocsf-data-lake/ — owned by your AWS account, not Databricks</text>

          <rect x="15" y="48" width="155" height="60" rx="5" fill="white" stroke="#a5d6a7" strokeWidth="1"/>
          <text x="92" y="66" textAnchor="middle" className="db-box-title">ocsf_class_4003</text>
          <text x="92" y="82" textAnchor="middle" className="db-item-dim">DNS Activity</text>
          <text x="92" y="96" textAnchor="middle" className="db-item-dim">Partitioned by date</text>

          <rect x="185" y="48" width="155" height="60" rx="5" fill="white" stroke="#a5d6a7" strokeWidth="1"/>
          <text x="262" y="66" textAnchor="middle" className="db-box-title">ocsf_class_3002</text>
          <text x="262" y="82" textAnchor="middle" className="db-item-dim">Authentication</text>
          <text x="262" y="96" textAnchor="middle" className="db-item-dim">Partitioned by date</text>

          <rect x="355" y="48" width="155" height="60" rx="5" fill="white" stroke="#a5d6a7" strokeWidth="1"/>
          <text x="432" y="66" textAnchor="middle" className="db-box-title">ocsf_observables</text>
          <text x="432" y="82" textAnchor="middle" className="db-item-dim">Hot path IOC matching</text>
          <text x="432" y="96" textAnchor="middle" className="db-item-dim">ZORDER by type, value</text>

          <rect x="525" y="48" width="160" height="60" rx="5" fill="#f1f8e9" stroke="#aed581" strokeWidth="1" strokeDasharray="4"/>
          <text x="605" y="66" textAnchor="middle" className="db-box-title">_semantic_index/</text>
          <text x="605" y="82" textAnchor="middle" className="db-item-dim">Index tables as Delta</text>
          <text x="605" y="96" textAnchor="middle" className="db-item-dim">(optional mirror)</text>

          <rect x="15" y="115" width="670" height="18" rx="3" fill="#f1f8e9" stroke="#aed581" strokeWidth="1" strokeDasharray="4"/>
          <text x="350" y="128" textAnchor="middle" className="db-item-dim">Parquet files on S3 · No Databricks compute for writes · Standard Delta protocol</text>
        </g>

        {/* ── Cost Savings Callout ── */}
        <g transform="translate(740, 400)">
          <rect x="0" y="0" width="200" height="130" rx="8" fill="#fff8e1" stroke="#ffc107" strokeWidth="2" filter="url(#dbShadow)"/>
          <text x="100" y="22" textAnchor="middle" className="db-section-title" fill="#f57f17">💰 Cost Savings</text>
          <text x="100" y="44" textAnchor="middle" className="db-item">No Databricks compute</text>
          <text x="100" y="58" textAnchor="middle" className="db-item">for ETL ingestion</text>
          <text x="100" y="78" textAnchor="middle" className="db-item">Rust container: ~$50/mo</text>
          <text x="100" y="92" textAnchor="middle" className="db-item-dim">vs Databricks Jobs:</text>
          <text x="100" y="106" textAnchor="middle" className="db-item-dim">~$500-2000/mo</text>
          <text x="100" y="122" textAnchor="middle" className="db-item" fill="#2e7d32">Up to 90% savings</text>
        </g>

        {/* Arrow: S3 → Databricks */}
        <line x1="350" y1="530" x2="350" y2="565" stroke="#ff3621" strokeWidth="2" markerEnd="url(#dbArrow)"/>
        <text x="400" y="552" className="db-arrow-label" fill="#ff3621">external tables</text>

        {/* Arrow: Editor → Databricks deploy */}
        <path d="M 845 350 L 845 565" fill="none" stroke="#667eea" strokeWidth="2" markerEnd="url(#dbArrowBlue)"/>
        <text x="870" y="460" className="db-arrow-label" fill="#667eea" transform="rotate(90, 870, 460)">deploy views</text>

        {/* ── Databricks Workspace (query-only) ── */}
        <g transform="translate(20, 570)">
          <rect x="0" y="0" width="920" height="470" rx="12" fill="#fafafa" stroke="#ff3621" strokeWidth="2" filter="url(#dbShadow)"/>
          <rect x="0" y="0" width="920" height="35" rx="12" fill="#ff3621"/>
          <rect x="0" y="18" width="920" height="17" fill="#ff3621"/>
          <text x="460" y="24" textAnchor="middle" fill="white" className="db-workspace-title">Databricks Workspace — Query &amp; Analytics Only</text>

          {/* ── External Tables (pointing to S3) ── */}
          <g transform="translate(20, 50)">
            <rect x="0" y="0" width="540" height="100" rx="8" fill="#e8f5e9" stroke="#66bb6a" strokeWidth="1.5"/>
            <text x="270" y="22" textAnchor="middle" className="db-section-title" fill="#2e7d32">Unity Catalog — External Tables (point to S3)</text>

            <rect x="15" y="38" width="155" height="48" rx="5" fill="white" stroke="#a5d6a7" strokeWidth="1"/>
            <text x="92" y="56" textAnchor="middle" className="db-box-title">ocsf_class_4003</text>
            <text x="92" y="72" textAnchor="middle" className="db-item-dim">LOCATION s3://...</text>

            <rect x="185" y="38" width="155" height="48" rx="5" fill="white" stroke="#a5d6a7" strokeWidth="1"/>
            <text x="262" y="56" textAnchor="middle" className="db-box-title">ocsf_class_3002</text>
            <text x="262" y="72" textAnchor="middle" className="db-item-dim">LOCATION s3://...</text>

            <rect x="355" y="38" width="170" height="48" rx="5" fill="white" stroke="#a5d6a7" strokeWidth="1"/>
            <text x="440" y="56" textAnchor="middle" className="db-box-title">ocsf_observables</text>
            <text x="440" y="72" textAnchor="middle" className="db-item-dim">LOCATION s3://...</text>
          </g>

          {/* ── Semantic Views (generated) ── */}
          <g transform="translate(20, 170)">
            <rect x="0" y="0" width="540" height="120" rx="8" fill="rgba(102,126,234,0.08)" stroke="#667eea" strokeWidth="2"/>
            <text x="270" y="22" textAnchor="middle" className="db-section-title" fill="#4c51bf">Semantic Layer — Generated SQL Views</text>

            <rect x="15" y="38" width="155" height="48" rx="5" fill="white" stroke="#667eea" strokeWidth="1.5"/>
            <text x="92" y="56" textAnchor="middle" className="db-box-title">v_dns_event</text>
            <text x="92" y="72" textAnchor="middle" className="db-item-dim">query_hostname, source_ip</text>

            <rect x="185" y="38" width="155" height="48" rx="5" fill="white" stroke="#667eea" strokeWidth="1.5"/>
            <text x="262" y="56" textAnchor="middle" className="db-box-title">v_login_event</text>
            <text x="262" y="72" textAnchor="middle" className="db-item-dim">user_name, status</text>

            <rect x="355" y="38" width="170" height="48" rx="5" fill="white" stroke="#667eea" strokeWidth="1.5"/>
            <text x="440" y="56" textAnchor="middle" className="db-box-title">dbt + Cube.js</text>
            <text x="440" y="72" textAnchor="middle" className="db-item-dim">Materialized metrics</text>

            <rect x="15" y="93" width="510" height="20" rx="3" fill="#ede7f6" stroke="#b39ddb" strokeWidth="1"/>
            <text x="270" y="107" textAnchor="middle" className="db-item-dim">Business-friendly names · Flattened JSON · 27 synonyms</text>
          </g>

          {/* ── Index-aware query optimization ── */}
          <g transform="translate(580, 50)">
            <rect x="0" y="0" width="320" height="240" rx="8" fill="rgba(56,178,172,0.08)" stroke="#38b2ac" strokeWidth="1.5" strokeDasharray="4"/>
            <text x="160" y="22" textAnchor="middle" className="db-section-title" fill="#285e61">Index-Aware Optimization</text>
            <text x="160" y="40" textAnchor="middle" className="db-item-dim">Reads from external PostgreSQL index</text>

            <rect x="15" y="52" width="135" height="42" rx="5" fill="white" stroke="#38b2ac" strokeWidth="1"/>
            <text x="82" y="70" textAnchor="middle" className="db-box-title">Partition Pruning</text>
            <text x="82" y="86" textAnchor="middle" className="db-item-dim">time bounds skip scans</text>

            <rect x="165" y="52" width="140" height="42" rx="5" fill="white" stroke="#38b2ac" strokeWidth="1"/>
            <text x="235" y="70" textAnchor="middle" className="db-box-title">Column Stats</text>
            <text x="235" y="86" textAnchor="middle" className="db-item-dim">min/max for planning</text>

            <rect x="15" y="105" width="135" height="42" rx="5" fill="white" stroke="#38b2ac" strokeWidth="1"/>
            <text x="82" y="123" textAnchor="middle" className="db-box-title">Lineage Lookup</text>
            <text x="82" y="139" textAnchor="middle" className="db-item-dim">field to source mapping</text>

            <rect x="165" y="105" width="140" height="42" rx="5" fill="white" stroke="#38b2ac" strokeWidth="1"/>
            <text x="235" y="123" textAnchor="middle" className="db-box-title">Query Cache</text>
            <text x="235" y="139" textAnchor="middle" className="db-item-dim">repeated query speedup</text>

            <rect x="15" y="160" width="290" height="65" rx="5" fill="#e0f2f1" stroke="#80cbc4" strokeWidth="1"/>
            <text x="160" y="180" textAnchor="middle" className="db-item">Databricks reads index metadata</text>
            <text x="160" y="196" textAnchor="middle" className="db-item">via JDBC to PostgreSQL</text>
            <text x="160" y="212" textAnchor="middle" className="db-item-dim">No Databricks storage cost for index</text>
          </g>

          {/* Arrow: External Tables to Views */}
          <line x1="270" y1="150" x2="270" y2="170" stroke="#667eea" strokeWidth="2" markerEnd="url(#dbArrowBlue)"/>

          {/* Arrow: Index to Views */}
          <path d="M 580 240 Q 560 260 560 290" fill="none" stroke="#38b2ac" strokeWidth="1.5" strokeDasharray="4" markerEnd="url(#dbArrowTeal)"/>
          <text x="590" y="275" className="db-arrow-label" fill="#38b2ac">optimizes</text>

          {/* ── Business Queries ── */}
          <g transform="translate(20, 310)">
            <rect x="0" y="0" width="880" height="100" rx="8" fill="#f3e5f5" stroke="#ba68c8" strokeWidth="1.5"/>
            <text x="440" y="20" textAnchor="middle" className="db-section-title" fill="#7b1fa2">Business Queries — Databricks SQL / Notebooks</text>

            <g transform="translate(15, 30)">
              <rect x="0" y="0" width="270" height="58" rx="5" fill="white" stroke="#e1bee7" strokeWidth="1"/>
              <text x="135" y="16" textAnchor="middle" className="db-box-title">Analyst Query</text>
              <text x="135" y="32" textAnchor="middle" className="db-code">SELECT query_hostname, source_ip</text>
              <text x="135" y="46" textAnchor="middle" className="db-code">FROM v_dns_event WHERE ...</text>
            </g>

            <g transform="translate(305, 30)">
              <rect x="0" y="0" width="270" height="58" rx="5" fill="white" stroke="#e1bee7" strokeWidth="1"/>
              <text x="135" y="16" textAnchor="middle" className="db-box-title">Threat Hunt (Hot Path)</text>
              <text x="135" y="32" textAnchor="middle" className="db-code">SELECT o.value, ti.threat_type</text>
              <text x="135" y="46" textAnchor="middle" className="db-code">FROM ocsf_observables o JOIN ...</text>
            </g>

            <g transform="translate(595, 30)">
              <rect x="0" y="0" width="270" height="58" rx="5" fill="white" stroke="#e1bee7" strokeWidth="1"/>
              <text x="135" y="16" textAnchor="middle" className="db-box-title">Metric Dashboard</text>
              <text x="135" y="32" textAnchor="middle" className="db-code">SELECT source_ip, nxdomain_rate</text>
              <text x="135" y="46" textAnchor="middle" className="db-code">FROM v_dns_event GROUP BY ...</text>
            </g>
          </g>

          {/* Arrow: Views to Queries */}
          <line x1="270" y1="290" x2="270" y2="310" stroke="#ba68c8" strokeWidth="2" markerEnd="url(#dbArrow)"/>

          {/* User personas */}
          <g transform="translate(20, 420)">
            <text x="110" y="12" textAnchor="middle" className="sl-persona">👤 Security Analyst</text>
            <text x="350" y="12" textAnchor="middle" className="sl-persona">🛡️ Threat Hunter</text>
            <text x="590" y="12" textAnchor="middle" className="sl-persona">📊 SOC Dashboard</text>
            <text x="810" y="12" textAnchor="middle" className="sl-persona">🔧 Data Engineer</text>
          </g>
        </g>
      </svg>

      <div className="diagram-legend">
        <h4>External ETL + Databricks Architecture</h4>
        <ul>
          <li><strong>Raw Sources → External Rust ETL:</strong> Raw security logs (Zeek, CloudTrail, VPC Flow, Syslog) are consumed by a lightweight Rust container running outside Databricks. Uses ocsf-core for OCSF normalization, ocsf-warehouse for Delta Lake writes, and ocsf-catalog for table registration. Runs on ECS, EKS, or bare metal at a fraction of Databricks compute cost.</li>
          <li><strong>S3 Delta Lake (Physical Storage):</strong> OCSF-normalized data is written directly to S3 as Delta Lake tables. Each event class gets its own table (ocsf_class_4003 for DNS, ocsf_class_3002 for Auth). Observables are extracted for hot path IOC matching. No Databricks compute is consumed for writes.</li>
          <li><strong>Semantic Index (PostgreSQL):</strong> Metadata lives in an external PostgreSQL instance — table registry, source/field lineage, partition bounds, and column statistics. Populated by the ETL container on each run. Databricks reads it via JDBC for query optimization. No Databricks storage cost for index data.</li>
          <li><strong>OCSF Semantic Editor:</strong> Authors the semantic-model.yaml and generates SQL views, dbt models, and Cube.js schemas. Artifacts are deployed to Databricks via CI/CD.</li>
          <li><strong>Databricks (Query Only):</strong> Unity Catalog registers external tables pointing to S3. Generated semantic views flatten nested JSON into business-friendly columns. Databricks is used exclusively for query and analytics — no ETL compute. This can reduce Databricks costs by up to 90%.</li>
          <li><strong>💰 Cost Impact:</strong> Moving ETL outside Databricks to a Rust container (~$50/mo) vs Databricks Jobs (~$500-2000/mo) dramatically reduces compute spend while maintaining full query capability through external tables.</li>
        </ul>
      </div>
    </div>
  );
}

function OverviewDiagram() {
  return (
    <div className="diagram-container">
      <svg viewBox="0 0 900 600" className="architecture-svg">
        {/* Background */}
        <defs>
          <linearGradient id="headerGrad" x1="0%" y1="0%" x2="100%" y2="0%">
            <stop offset="0%" stopColor="#667eea" />
            <stop offset="100%" stopColor="#764ba2" />
          </linearGradient>
          <filter id="shadow" x="-20%" y="-20%" width="140%" height="140%">
            <feDropShadow dx="2" dy="2" stdDeviation="3" floodOpacity="0.2"/>
          </filter>
        </defs>

        {/* Title */}
        <text x="450" y="35" textAnchor="middle" className="diagram-title">
          OCSF Semantic Layer Architecture
        </text>

        {/* Layer Labels */}
        <text x="50" y="90" className="layer-label">Presentation</text>
        <text x="50" y="220" className="layer-label">API</text>
        <text x="50" y="350" className="layer-label">Core</text>
        <text x="50" y="480" className="layer-label">Storage</text>

        {/* Presentation Layer */}
        <g transform="translate(150, 60)">
          <rect x="0" y="0" width="700" height="80" rx="8" className="layer-box presentation" filter="url(#shadow)"/>
          <rect x="20" y="15" width="150" height="50" rx="4" className="component-box"/>
          <text x="95" y="45" textAnchor="middle" className="component-text">Editor UI</text>
          <rect x="190" y="15" width="150" height="50" rx="4" className="component-box"/>
          <text x="265" y="45" textAnchor="middle" className="component-text">Schema Browser</text>
          <rect x="360" y="15" width="150" height="50" rx="4" className="component-box"/>
          <text x="435" y="45" textAnchor="middle" className="component-text">Lineage Viz</text>
          <rect x="530" y="15" width="150" height="50" rx="4" className="component-box"/>
          <text x="605" y="45" textAnchor="middle" className="component-text">Coverage Dashboard</text>
        </g>

        {/* API Layer */}
        <g transform="translate(150, 190)">
          <rect x="0" y="0" width="700" height="80" rx="8" className="layer-box api" filter="url(#shadow)"/>
          <rect x="20" y="15" width="120" height="50" rx="4" className="component-box"/>
          <text x="80" y="45" textAnchor="middle" className="component-text">REST API</text>
          <rect x="160" y="15" width="120" height="50" rx="4" className="component-box"/>
          <text x="220" y="45" textAnchor="middle" className="component-text">Schema API</text>
          <rect x="300" y="15" width="120" height="50" rx="4" className="component-box"/>
          <text x="360" y="45" textAnchor="middle" className="component-text">Index API</text>
          <rect x="440" y="15" width="120" height="50" rx="4" className="component-box"/>
          <text x="500" y="45" textAnchor="middle" className="component-text">LLM API</text>
          <rect x="580" y="15" width="100" height="50" rx="4" className="component-box"/>
          <text x="630" y="45" textAnchor="middle" className="component-text">Validation</text>
        </g>

        {/* Core Layer */}
        <g transform="translate(150, 320)">
          <rect x="0" y="0" width="700" height="80" rx="8" className="layer-box core" filter="url(#shadow)"/>
          <rect x="20" y="15" width="130" height="50" rx="4" className="component-box highlight"/>
          <text x="85" y="45" textAnchor="middle" className="component-text">Semantic Index</text>
          <rect x="170" y="15" width="130" height="50" rx="4" className="component-box"/>
          <text x="235" y="45" textAnchor="middle" className="component-text">Table Registry</text>
          <rect x="320" y="15" width="130" height="50" rx="4" className="component-box"/>
          <text x="385" y="45" textAnchor="middle" className="component-text">Lineage Store</text>
          <rect x="470" y="15" width="100" height="50" rx="4" className="component-box"/>
          <text x="520" y="45" textAnchor="middle" className="component-text">Coverage</text>
          <rect x="590" y="15" width="90" height="50" rx="4" className="component-box"/>
          <text x="635" y="45" textAnchor="middle" className="component-text">Statistics</text>
        </g>

        {/* Storage Layer */}
        <g transform="translate(150, 450)">
          <rect x="0" y="0" width="700" height="80" rx="8" className="layer-box storage" filter="url(#shadow)"/>
          <rect x="80" y="15" width="160" height="50" rx="4" className="component-box storage-box"/>
          <text x="160" y="45" textAnchor="middle" className="component-text">SQLite Backend</text>
          <rect x="280" y="15" width="160" height="50" rx="4" className="component-box storage-box"/>
          <text x="360" y="45" textAnchor="middle" className="component-text">In-Memory Backend</text>
          <rect x="480" y="15" width="160" height="50" rx="4" className="component-box storage-box"/>
          <text x="560" y="45" textAnchor="middle" className="component-text">OCSF Schema Files</text>
        </g>

        {/* Arrows */}
        <g className="arrows">
          <line x1="500" y1="140" x2="500" y2="190" markerEnd="url(#arrowhead)"/>
          <line x1="500" y1="270" x2="500" y2="320" markerEnd="url(#arrowhead)"/>
          <line x1="500" y1="400" x2="500" y2="450" markerEnd="url(#arrowhead)"/>
        </g>

        {/* Arrow marker */}
        <defs>
          <marker id="arrowhead" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
            <polygon points="0 0, 10 3.5, 0 7" fill="#666"/>
          </marker>
        </defs>
      </svg>
    </div>
  );
}

function DataFlowDiagram() {
  return (
    <div className="diagram-container">
      <svg viewBox="0 0 900 550" className="architecture-svg">
        <text x="450" y="35" textAnchor="middle" className="diagram-title">
          Data Flow: Raw Logs → OCSF → Analytics
        </text>

        {/* Raw Data Sources */}
        <g transform="translate(50, 80)">
          <rect x="0" y="0" width="140" height="60" rx="6" className="source-box"/>
          <text x="70" y="25" textAnchor="middle" className="box-label">Raw Logs</text>
          <text x="70" y="45" textAnchor="middle" className="box-sublabel">Zeek, Suricata, etc.</text>
        </g>

        <g transform="translate(50, 160)">
          <rect x="0" y="0" width="140" height="60" rx="6" className="source-box"/>
          <text x="70" y="25" textAnchor="middle" className="box-label">Cloud Logs</text>
          <text x="70" y="45" textAnchor="middle" className="box-sublabel">AWS, Azure, GCP</text>
        </g>

        <g transform="translate(50, 240)">
          <rect x="0" y="0" width="140" height="60" rx="6" className="source-box"/>
          <text x="70" y="25" textAnchor="middle" className="box-label">EDR Data</text>
          <text x="70" y="45" textAnchor="middle" className="box-sublabel">CrowdStrike, etc.</text>
        </g>

        {/* ETL/Transform */}
        <g transform="translate(250, 140)">
          <rect x="0" y="0" width="160" height="100" rx="8" className="transform-box"/>
          <text x="80" y="30" textAnchor="middle" className="box-label">ETL Pipeline</text>
          <text x="80" y="55" textAnchor="middle" className="box-sublabel">Field Mapping</text>
          <text x="80" y="75" textAnchor="middle" className="box-sublabel">Normalization</text>
        </g>

        {/* Semantic Index */}
        <g transform="translate(470, 100)">
          <rect x="0" y="0" width="180" height="180" rx="8" className="index-box"/>
          <text x="90" y="30" textAnchor="middle" className="box-label">Semantic Index</text>
          
          <rect x="15" y="50" width="150" height="35" rx="4" className="inner-box"/>
          <text x="90" y="72" textAnchor="middle" className="inner-label">Table Registry</text>
          
          <rect x="15" y="95" width="150" height="35" rx="4" className="inner-box"/>
          <text x="90" y="117" textAnchor="middle" className="inner-label">Source Lineage</text>
          
          <rect x="15" y="140" width="150" height="35" rx="4" className="inner-box"/>
          <text x="90" y="162" textAnchor="middle" className="inner-label">Field Lineage</text>
        </g>

        {/* OCSF Tables */}
        <g transform="translate(710, 80)">
          <rect x="0" y="0" width="140" height="50" rx="6" className="ocsf-box"/>
          <text x="70" y="30" textAnchor="middle" className="box-label">DNS Activity</text>
        </g>

        <g transform="translate(710, 145)">
          <rect x="0" y="0" width="140" height="50" rx="6" className="ocsf-box"/>
          <text x="70" y="30" textAnchor="middle" className="box-label">Network Activity</text>
        </g>

        <g transform="translate(710, 210)">
          <rect x="0" y="0" width="140" height="50" rx="6" className="ocsf-box"/>
          <text x="70" y="30" textAnchor="middle" className="box-label">Authentication</text>
        </g>

        {/* Analytics */}
        <g transform="translate(470, 340)">
          <rect x="0" y="0" width="180" height="120" rx="8" className="analytics-box"/>
          <text x="90" y="30" textAnchor="middle" className="box-label">Analytics Layer</text>
          
          <rect x="15" y="50" width="150" height="25" rx="4" className="inner-box"/>
          <text x="90" y="67" textAnchor="middle" className="inner-label">Detection Coverage</text>
          
          <rect x="15" y="85" width="150" height="25" rx="4" className="inner-box"/>
          <text x="90" y="102" textAnchor="middle" className="inner-label">Statistics</text>
        </g>

        {/* Arrows */}
        <defs>
          <marker id="arrow" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
            <polygon points="0 0, 10 3.5, 0 7" fill="#667eea"/>
          </marker>
        </defs>

        {/* Flow arrows */}
        <g className="flow-arrows">
          <path d="M 190 110 Q 220 110 220 160 L 220 190 L 250 190" fill="none" stroke="#667eea" strokeWidth="2" markerEnd="url(#arrow)"/>
          <path d="M 190 190 L 250 190" fill="none" stroke="#667eea" strokeWidth="2" markerEnd="url(#arrow)"/>
          <path d="M 190 270 Q 220 270 220 220 L 220 190 L 250 190" fill="none" stroke="#667eea" strokeWidth="2" markerEnd="url(#arrow)"/>
          
          <line x1="410" y1="190" x2="470" y2="190" stroke="#667eea" strokeWidth="2" markerEnd="url(#arrow)"/>
          <line x1="650" y1="190" x2="710" y2="190" stroke="#667eea" strokeWidth="2" markerEnd="url(#arrow)"/>
          <line x1="560" y1="280" x2="560" y2="340" stroke="#667eea" strokeWidth="2" markerEnd="url(#arrow)"/>
        </g>

        {/* Labels */}
        <text x="320" y="130" className="flow-label">Parse &amp;</text>
        <text x="320" y="145" className="flow-label">Transform</text>
        <text x="560" y="315" className="flow-label">Query</text>
        <text x="680" y="175" className="flow-label">Store</text>
      </svg>
    </div>
  );
}

function IndexStructureDiagram() {
  return (
    <div className="diagram-container">
      <svg viewBox="0 0 900 500" className="architecture-svg">
        <text x="450" y="35" textAnchor="middle" className="diagram-title">
          Semantic Index Structure
        </text>

        {/* Central Index */}
        <g transform="translate(350, 80)">
          <rect x="0" y="0" width="200" height="80" rx="8" className="central-box"/>
          <text x="100" y="35" textAnchor="middle" className="central-label">SemanticIndex</text>
          <text x="100" y="55" textAnchor="middle" className="central-sublabel">Central coordinator</text>
        </g>

        {/* Table Registry */}
        <g transform="translate(50, 220)">
          <rect x="0" y="0" width="180" height="120" rx="6" className="store-box registry"/>
          <text x="90" y="25" textAnchor="middle" className="store-label">Table Registry</text>
          <text x="90" y="50" textAnchor="middle" className="store-field">• table_name</text>
          <text x="90" y="70" textAnchor="middle" className="store-field">• class_uid</text>
          <text x="90" y="90" textAnchor="middle" className="store-field">• ocsf_version</text>
          <text x="90" y="110" textAnchor="middle" className="store-field">• detection_coverage</text>
        </g>

        {/* Source Lineage */}
        <g transform="translate(260, 220)">
          <rect x="0" y="0" width="180" height="120" rx="6" className="store-box lineage"/>
          <text x="90" y="25" textAnchor="middle" className="store-label">Source Lineage</text>
          <text x="90" y="50" textAnchor="middle" className="store-field">• source_table</text>
          <text x="90" y="70" textAnchor="middle" className="store-field">• target_table</text>
          <text x="90" y="90" textAnchor="middle" className="store-field">• transformation_type</text>
          <text x="90" y="110" textAnchor="middle" className="store-field">• sql_expression</text>
        </g>

        {/* Field Lineage */}
        <g transform="translate(470, 220)">
          <rect x="0" y="0" width="180" height="120" rx="6" className="store-box field"/>
          <text x="90" y="25" textAnchor="middle" className="store-label">Field Lineage</text>
          <text x="90" y="50" textAnchor="middle" className="store-field">• source_field</text>
          <text x="90" y="70" textAnchor="middle" className="store-field">• target_field</text>
          <text x="90" y="90" textAnchor="middle" className="store-field">• transformation</text>
          <text x="90" y="110" textAnchor="middle" className="store-field">• confidence</text>
        </g>

        {/* Statistics */}
        <g transform="translate(680, 220)">
          <rect x="0" y="0" width="170" height="120" rx="6" className="store-box stats"/>
          <text x="85" y="25" textAnchor="middle" className="store-label">Statistics</text>
          <text x="85" y="50" textAnchor="middle" className="store-field">• row_count</text>
          <text x="85" y="70" textAnchor="middle" className="store-field">• column_stats</text>
          <text x="85" y="90" textAnchor="middle" className="store-field">• null_ratio</text>
          <text x="85" y="110" textAnchor="middle" className="store-field">• cardinality</text>
        </g>

        {/* Backend */}
        <g transform="translate(300, 400)">
          <rect x="0" y="0" width="300" height="70" rx="8" className="backend-box"/>
          <text x="150" y="30" textAnchor="middle" className="backend-label">IndexBackend Trait</text>
          <text x="150" y="50" textAnchor="middle" className="backend-sublabel">SQLite | In-Memory</text>
        </g>

        {/* Connecting lines */}
        <defs>
          <marker id="arrow2" markerWidth="8" markerHeight="6" refX="7" refY="3" orient="auto">
            <polygon points="0 0, 8 3, 0 6" fill="#888"/>
          </marker>
        </defs>

        <g className="connectors">
          <line x1="450" y1="160" x2="140" y2="220" stroke="#888" strokeWidth="1.5" markerEnd="url(#arrow2)"/>
          <line x1="450" y1="160" x2="350" y2="220" stroke="#888" strokeWidth="1.5" markerEnd="url(#arrow2)"/>
          <line x1="450" y1="160" x2="560" y2="220" stroke="#888" strokeWidth="1.5" markerEnd="url(#arrow2)"/>
          <line x1="450" y1="160" x2="765" y2="220" stroke="#888" strokeWidth="1.5" markerEnd="url(#arrow2)"/>
          
          <line x1="140" y1="340" x2="350" y2="400" stroke="#888" strokeWidth="1.5" strokeDasharray="4"/>
          <line x1="350" y1="340" x2="400" y2="400" stroke="#888" strokeWidth="1.5" strokeDasharray="4"/>
          <line x1="560" y1="340" x2="500" y2="400" stroke="#888" strokeWidth="1.5" strokeDasharray="4"/>
          <line x1="765" y1="340" x2="550" y2="400" stroke="#888" strokeWidth="1.5" strokeDasharray="4"/>
        </g>
      </svg>

      <div className="diagram-legend">
        <h4>Key Concepts</h4>
        <ul>
          <li><strong>Table Registry:</strong> Tracks OCSF tables with their class mappings and detection coverage metadata</li>
          <li><strong>Source Lineage:</strong> Records table-to-table transformations (ETL pipelines)</li>
          <li><strong>Field Lineage:</strong> Tracks field-level mappings with SQL transformations</li>
          <li><strong>Statistics:</strong> Column-level statistics for query optimization</li>
        </ul>
      </div>
    </div>
  );
}

export default ArchitectureDiagram;
