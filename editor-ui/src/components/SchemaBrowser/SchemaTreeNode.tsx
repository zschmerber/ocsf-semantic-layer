/**
 * SchemaTreeNode component for rendering expandable tree nodes.
 * 
 * Renders categories, classes, objects, and attributes as expandable tree nodes
 * with expand/collapse state and attribute metadata display on selection.
 * Supports drag-and-drop for attributes to the Entity Editor.
 * 
 * Requirements: 1.1, 1.2, 1.3, 1.4, 2.3, 2.4
 */

import { useCallback, memo } from 'react';
import { useDrag } from 'react-dnd';
import type { CategoryNode, ClassNode, ObjectNode, AttributeNode } from '../../types';
import './SchemaBrowser.css';

// ============================================
// Types
// ============================================

export type SchemaNodeType = 'category' | 'class' | 'object' | 'attribute';

export interface SchemaNodeData {
  id: string;
  type: SchemaNodeType;
  name: string;
  caption: string;
  description: string;
  metadata?: AttributeMetadata;
  children?: SchemaNodeData[];
  /** Full OCSF path for attributes (used for drag-drop) */
  ocsfPath?: string;
}

export interface AttributeMetadata {
  attrType: string;
  typeName: string;
  requirement: string;
  isArray: boolean;
  enumValues?: Array<{ key: string; caption: string; description?: string }>;
  objectType?: string;
}

interface SchemaTreeNodeProps {
  node: SchemaNodeData;
  depth: number;
  isExpanded: boolean;
  isSelected: boolean;
  searchQuery: string;
  expandedNodes: Set<string>;
  onToggle: (nodeId: string) => void;
  onSelect: (node: SchemaNodeData) => void;
}

// Drag item type for OCSF attributes (shared with AttributeMappingZone)
export const OCSF_ATTRIBUTE_TYPE = 'ocsf-attribute';

export interface OCSFAttributeDragItem {
  type: typeof OCSF_ATTRIBUTE_TYPE;
  path: string;
  name: string;
  caption: string;
  description: string;
  attrType: string;
  isArray: boolean;
}

// ============================================
// Helper Functions
// ============================================

/**
 * Highlights matching text in a string based on search query.
 */
function highlightMatch(text: string, query: string): React.ReactNode {
  if (!query) return text;
  
  const lowerText = text.toLowerCase();
  const lowerQuery = query.toLowerCase();
  const index = lowerText.indexOf(lowerQuery);
  
  if (index === -1) return text;
  
  return (
    <>
      {text.slice(0, index)}
      <mark className="search-highlight">{text.slice(index, index + query.length)}</mark>
      {text.slice(index + query.length)}
    </>
  );
}

/**
 * Gets the icon for a node type.
 */
function getNodeIcon(type: SchemaNodeType): string {
  switch (type) {
    case 'category':
      return '📁';
    case 'class':
      return '📋';
    case 'object':
      return '📦';
    case 'attribute':
      return '🔹';
    default:
      return '•';
  }
}

/**
 * Gets the CSS class for a node type.
 */
function getNodeTypeClass(type: SchemaNodeType): string {
  return `node-type-${type}`;
}

// ============================================
// Component
// ============================================

/**
 * SchemaTreeNode renders a single node in the OCSF schema tree.
 * 
 * Features:
 * - Expandable/collapsible nodes for categories, classes, and objects
 * - Attribute metadata display on selection
 * - Search query highlighting
 * - Keyboard navigation support
 * - Drag support for attributes (to Entity Editor)
 */
export const SchemaTreeNode = memo(function SchemaTreeNode({
  node,
  depth,
  isExpanded,
  isSelected,
  searchQuery,
  expandedNodes,
  onToggle,
  onSelect,
}: SchemaTreeNodeProps) {
  const hasChildren = node.children && node.children.length > 0;
  const isDraggable = node.type === 'attribute' && node.ocsfPath;
  
  // Set up drag for attribute nodes
  const [{ isDragging }, dragRef] = useDrag<
    OCSFAttributeDragItem,
    unknown,
    { isDragging: boolean }
  >(
    () => ({
      type: OCSF_ATTRIBUTE_TYPE,
      item: {
        type: OCSF_ATTRIBUTE_TYPE,
        path: node.ocsfPath || '',
        name: node.name,
        caption: node.caption,
        description: node.description,
        attrType: node.metadata?.attrType ?? 'string',
        isArray: node.metadata?.isArray ?? false,
      },
      canDrag: () => !!isDraggable && !!node.ocsfPath,
      collect: (monitor) => ({
        isDragging: monitor.isDragging(),
      }),
    }),
    [node, isDraggable]
  );
  
  const handleToggle = useCallback((e: React.MouseEvent) => {
    e.stopPropagation();
    if (hasChildren) {
      onToggle(node.id);
    }
  }, [hasChildren, node.id, onToggle]);
  
  const handleSelect = useCallback(() => {
    onSelect(node);
  }, [node, onSelect]);
  
  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      handleSelect();
    } else if (e.key === 'ArrowRight' && hasChildren && !isExpanded) {
      e.preventDefault();
      onToggle(node.id);
    } else if (e.key === 'ArrowLeft' && hasChildren && isExpanded) {
      e.preventDefault();
      onToggle(node.id);
    }
  }, [handleSelect, hasChildren, isExpanded, node.id, onToggle]);

  const nodeClassName = [
    'tree-node',
    isSelected ? 'selected' : '',
    getNodeTypeClass(node.type),
    isDraggable ? 'draggable' : '',
    isDragging ? 'dragging' : '',
  ].filter(Boolean).join(' ');

  return (
    <div className="tree-node-container">
      <div
        ref={isDraggable ? dragRef : undefined}
        className={nodeClassName}
        style={{ paddingLeft: `${depth * 16 + 8}px` }}
        onClick={handleSelect}
        onKeyDown={handleKeyDown}
        role="treeitem"
        aria-expanded={hasChildren ? isExpanded : undefined}
        aria-selected={isSelected}
        tabIndex={0}
      >
        {hasChildren ? (
          <button
            className={`expand-btn ${isExpanded ? 'expanded' : ''}`}
            onClick={handleToggle}
            aria-label={isExpanded ? 'Collapse' : 'Expand'}
          >
            <span className="expand-icon">▶</span>
          </button>
        ) : (
          <span className="expand-placeholder" />
        )}
        
        <span className="node-icon">{getNodeIcon(node.type)}</span>
        <span className="node-name">{highlightMatch(node.name, searchQuery)}</span>
        
        {node.metadata && (
          <span className="node-type-badge">{node.metadata.typeName}</span>
        )}
        
        {node.metadata?.requirement && (
          <span className={`requirement-badge ${node.metadata.requirement}`}>
            {node.metadata.requirement}
          </span>
        )}
        
        {isDraggable && (
          <span className="drag-hint" title="Drag to Entity Editor">⋮⋮</span>
        )}
      </div>
      
      {hasChildren && isExpanded && (
        <div className="tree-children" role="group">
          {node.children!.map((child) => (
            <SchemaTreeNode
              key={child.id}
              node={child}
              depth={depth + 1}
              isExpanded={expandedNodes.has(child.id)}
              isSelected={false}
              searchQuery={searchQuery}
              expandedNodes={expandedNodes}
              onToggle={onToggle}
              onSelect={onSelect}
            />
          ))}
        </div>
      )}
    </div>
  );
});

// ============================================
// Conversion Functions
// ============================================

/**
 * Converts a CategoryNode to SchemaNodeData.
 */
export function categoryToNode(category: CategoryNode): SchemaNodeData {
  return {
    id: `category-${category.uid}`,
    type: 'category',
    name: category.name,
    caption: category.caption,
    description: category.description,
    children: category.classes.map(classToNode),
  };
}

/**
 * Converts a ClassNode to SchemaNodeData.
 */
export function classToNode(cls: ClassNode): SchemaNodeData {
  return {
    id: `class-${cls.uid}`,
    type: 'class',
    name: cls.name,
    caption: cls.caption,
    description: cls.description,
    children: cls.attributes.map((attr) => attributeToNode(attr, `class-${cls.uid}`, '')),
  };
}

/**
 * Converts an ObjectNode to SchemaNodeData.
 */
export function objectToNode(obj: ObjectNode): SchemaNodeData {
  return {
    id: `object-${obj.name}`,
    type: 'object',
    name: obj.name,
    caption: obj.caption,
    description: obj.description,
    children: obj.attributes.map((attr) => attributeToNode(attr, `object-${obj.name}`, '')),
  };
}

/**
 * Converts an AttributeNode to SchemaNodeData.
 * @param attr The attribute node to convert
 * @param parentId The parent node ID for generating unique IDs
 * @param parentPath The parent OCSF path (empty for top-level attributes)
 */
export function attributeToNode(attr: AttributeNode, parentId: string, parentPath: string): SchemaNodeData {
  const id = `${parentId}-attr-${attr.name}`;
  const ocsfPath = parentPath ? `${parentPath}.${attr.name}` : attr.name;
  
  return {
    id,
    type: 'attribute',
    name: attr.name,
    caption: attr.caption,
    description: attr.description,
    ocsfPath,
    metadata: {
      attrType: attr.type,
      typeName: attr.type,
      requirement: attr.requirement,
      isArray: attr.is_array,
      enumValues: attr.enum_values,
      objectType: attr.object_type,
    },
    children: attr.children?.map((child) => attributeToNode(child, id, ocsfPath)),
  };
}

export default SchemaTreeNode;
