/**
 * AttributeDetails component for displaying attribute metadata.
 * 
 * Shows detailed metadata when an attribute is selected including:
 * - Type information
 * - Requirement level
 * - Enum values (if applicable)
 * - Description
 * 
 * Requirements: 1.4
 */

import type { SchemaNodeData, AttributeMetadata } from './SchemaTreeNode';
import './SchemaBrowser.css';

interface AttributeDetailsProps {
  node: SchemaNodeData | null;
}

/**
 * Renders the requirement badge with appropriate styling.
 */
function RequirementBadge({ requirement }: { requirement: string }) {
  return (
    <span className={`detail-badge ${requirement}`}>
      {requirement}
    </span>
  );
}

/**
 * Renders enum values in a list format.
 */
function EnumValuesList({ values }: { values: NonNullable<AttributeMetadata['enumValues']> }) {
  if (values.length === 0) return null;
  
  return (
    <div className="detail-section">
      <h4 className="detail-section-title">Enum Values</h4>
      <div className="enum-values-list">
        {values.map((val) => (
          <div key={val.key} className="enum-value-item">
            <span className="enum-key">{val.key}</span>
            <span className="enum-caption">{val.caption}</span>
            {val.description && (
              <span className="enum-description">{val.description}</span>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}

/**
 * AttributeDetails displays detailed metadata for a selected schema node.
 */
export function AttributeDetails({ node }: AttributeDetailsProps) {
  if (!node) {
    return (
      <div className="attribute-details empty">
        <p className="empty-message">Select an item to view details</p>
      </div>
    );
  }

  const { metadata } = node;

  return (
    <div className="attribute-details">
      <div className="detail-header">
        <h3 className="detail-name">{node.caption || node.name}</h3>
        <span className="detail-type-label">{node.type}</span>
      </div>
      
      {node.description && (
        <p className="detail-description">{node.description}</p>
      )}
      
      {metadata && (
        <div className="detail-metadata">
          <div className="detail-row">
            <span className="detail-label">Type:</span>
            <span className="detail-value mono">{metadata.typeName}</span>
            {metadata.isArray && <span className="detail-badge array">array</span>}
          </div>
          
          <div className="detail-row">
            <span className="detail-label">Requirement:</span>
            <RequirementBadge requirement={metadata.requirement} />
          </div>
          
          {metadata.objectType && (
            <div className="detail-row">
              <span className="detail-label">Object Type:</span>
              <span className="detail-value mono">{metadata.objectType}</span>
            </div>
          )}
          
          {metadata.enumValues && metadata.enumValues.length > 0 && (
            <EnumValuesList values={metadata.enumValues} />
          )}
        </div>
      )}
      
      {!metadata && node.type !== 'attribute' && (
        <div className="detail-metadata">
          <div className="detail-row">
            <span className="detail-label">Name:</span>
            <span className="detail-value mono">{node.name}</span>
          </div>
        </div>
      )}
    </div>
  );
}

export default AttributeDetails;
