import { useState } from 'react';
import './ArchitectureDiagram.css';

type DiagramView = 'overview' | 'dataflow' | 'index' | 'fullarch';

interface DiagramViewOption {
  id: DiagramView;
  label: string;
  description: string;
}

const viewOptions: DiagramViewOption[] = [
  { id: 'fullarch', label: 'Full Architecture', description: 'Complete system architecture diagram' },
  { id: 'overview', label: 'System Overview', description: 'High-level architecture of the OCSF Semantic Layer' },
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
