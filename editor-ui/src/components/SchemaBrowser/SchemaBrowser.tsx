/**
 * SchemaBrowser component for browsing OCSF schema hierarchy.
 * 
 * Displays OCSF categories, event classes, objects, and attributes
 * as an expandable tree with search functionality.
 * 
 * Now fetches schema from the backend API using TanStack Query.
 * 
 * Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 7.1
 */

import { useState, useCallback, useMemo, useEffect } from 'react';
import { useEditorStore } from '../../store';
import { useSchema, getErrorMessage } from '../../api';
import { SchemaTreeNode, categoryToNode, objectToNode } from './SchemaTreeNode';
import type { SchemaNodeData } from './SchemaTreeNode';
import { AttributeDetails } from './AttributeDetails';
import { SchemaSearch } from './SchemaSearch';
import { ContextualTooltip } from '../ContextualTooltip';
import './SchemaBrowser.css';

// ============================================
// Helper Functions
// ============================================

/**
 * Checks if a node or any of its descendants match the search query.
 */
function nodeMatchesSearch(node: SchemaNodeData, query: string): boolean {
  const lowerQuery = query.toLowerCase();
  
  // Check if this node matches
  if (
    node.name.toLowerCase().includes(lowerQuery) ||
    node.caption.toLowerCase().includes(lowerQuery) ||
    node.description.toLowerCase().includes(lowerQuery)
  ) {
    return true;
  }
  
  // Check if any children match
  if (node.children) {
    return node.children.some((child) => nodeMatchesSearch(child, query));
  }
  
  return false;
}

/**
 * Filters the tree to only include nodes that match the search query.
 * Preserves parent nodes that have matching descendants.
 */
function filterTree(nodes: SchemaNodeData[], query: string): SchemaNodeData[] {
  if (!query) return nodes;
  
  return nodes
    .filter((node) => nodeMatchesSearch(node, query))
    .map((node) => {
      if (!node.children) return node;
      
      const filteredChildren = filterTree(node.children, query);
      return {
        ...node,
        children: filteredChildren,
      };
    });
}

/**
 * Collects all node IDs that should be expanded to show search results.
 */
function getExpandedNodesForSearch(nodes: SchemaNodeData[], query: string): Set<string> {
  const expanded = new Set<string>();
  
  function collectExpanded(node: SchemaNodeData): boolean {
    let hasMatchingDescendant = false;
    
    if (node.children) {
      for (const child of node.children) {
        if (collectExpanded(child)) {
          hasMatchingDescendant = true;
        }
      }
    }
    
    const lowerQuery = query.toLowerCase();
    const nodeMatches =
      node.name.toLowerCase().includes(lowerQuery) ||
      node.caption.toLowerCase().includes(lowerQuery) ||
      node.description.toLowerCase().includes(lowerQuery);
    
    if (hasMatchingDescendant || nodeMatches) {
      if (node.children && node.children.length > 0) {
        expanded.add(node.id);
      }
      return true;
    }
    
    return false;
  }
  
  nodes.forEach(collectExpanded);
  return expanded;
}

// ============================================
// Component
// ============================================

export function SchemaBrowser() {
  // Store state
  const schema = useEditorStore((state) => state.schema);
  const storeExpandedNodes = useEditorStore((state) => state.expandedNodes);
  const searchQuery = useEditorStore((state) => state.searchQuery);
  const toggleNode = useEditorStore((state) => state.toggleNode);
  const setSearchQuery = useEditorStore((state) => state.setSearchQuery);
  const setSchema = useEditorStore((state) => state.setSchema);
  const setSchemaLoading = useEditorStore((state) => state.setSchemaLoading);
  const setSchemaError = useEditorStore((state) => state.setSchemaError);
  
  // Fetch schema from API using TanStack Query (Requirement 7.1)
  const { 
    data: apiSchema, 
    isLoading: apiLoading, 
    error: apiError,
    refetch: refetchSchema,
  } = useSchema();
  
  // Sync API schema to store when it changes
  useEffect(() => {
    if (apiSchema) {
      setSchema(apiSchema);
      setSchemaError(null);
    }
  }, [apiSchema, setSchema, setSchemaError]);
  
  // Sync loading state to store
  useEffect(() => {
    setSchemaLoading(apiLoading);
  }, [apiLoading, setSchemaLoading]);
  
  // Sync error state to store
  useEffect(() => {
    if (apiError) {
      setSchemaError(getErrorMessage(apiError));
    }
  }, [apiError, setSchemaError]);
  
  const [selectedNode, setSelectedNode] = useState<SchemaNodeData | null>(null);
  const [localExpandedNodes, setLocalExpandedNodes] = useState<Set<string>>(new Set());
  
  // Use schema from store (which is synced from API)
  const schemaLoading = useEditorStore((state) => state.schemaLoading);
  const schemaError = useEditorStore((state) => state.schemaError);
  
  // Convert schema to tree nodes
  const treeNodes = useMemo(() => {
    if (!schema) return [];
    
    const categoryNodes = schema.categories.map(categoryToNode);
    const objectNodes = schema.objects.map(objectToNode);
    
    // Group objects under a virtual "Objects" category
    const objectsCategory: SchemaNodeData = {
      id: 'objects-root',
      type: 'category',
      name: 'Objects',
      caption: 'Objects',
      description: 'Reusable object definitions',
      children: objectNodes,
    };
    
    return [...categoryNodes, objectsCategory];
  }, [schema]);
  
  // Filter tree based on search query
  const filteredNodes = useMemo(() => {
    return filterTree(treeNodes, searchQuery);
  }, [treeNodes, searchQuery]);
  
  // Compute expanded nodes - combine store state with search-based expansion
  const expandedNodes = useMemo(() => {
    if (searchQuery) {
      // When searching, auto-expand nodes to show results
      return getExpandedNodesForSearch(treeNodes, searchQuery);
    }
    // Merge store expanded nodes with local expanded nodes
    return new Set([...storeExpandedNodes, ...localExpandedNodes]);
  }, [searchQuery, treeNodes, storeExpandedNodes, localExpandedNodes]);
  
  const handleToggle = useCallback((nodeId: string) => {
    if (searchQuery) {
      // During search, use local state for expansion
      setLocalExpandedNodes((prev) => {
        const next = new Set(prev);
        if (next.has(nodeId)) {
          next.delete(nodeId);
        } else {
          next.add(nodeId);
        }
        return next;
      });
    } else {
      toggleNode(nodeId);
    }
  }, [searchQuery, toggleNode]);
  
  const handleSelect = useCallback((node: SchemaNodeData) => {
    setSelectedNode(node);
  }, []);
  
  const handleSearchChange = useCallback((query: string) => {
    setSearchQuery(query);
    // Clear local expanded nodes when search changes
    setLocalExpandedNodes(new Set());
  }, [setSearchQuery]);
  
  // Loading state
  if (schemaLoading) {
    return (
      <div className="schema-browser">
        <div className="schema-browser-loading">
          <div className="spinner" />
          <p>Loading schema...</p>
        </div>
      </div>
    );
  }
  
  // Error state
  if (schemaError) {
    return (
      <div className="schema-browser">
        <div className="schema-browser-error">
          <p className="error-message">{schemaError}</p>
          <button 
            className="btn primary retry-btn" 
            onClick={() => refetchSchema()}
          >
            Retry
          </button>
        </div>
      </div>
    );
  }
  
  // Empty state
  if (!schema) {
    return (
      <div className="schema-browser">
        <div className="schema-browser-empty">
          <p>No schema loaded</p>
          <p className="text-muted">Load an OCSF schema to browse categories and classes</p>
          <button 
            className="btn primary load-btn" 
            onClick={() => refetchSchema()}
          >
            Load Schema
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="schema-browser">
      <div className="schema-browser-header">
        <ContextualTooltip term="event_class">
          <span className="schema-browser-title">Event Classes</span>
        </ContextualTooltip>
        <SchemaSearch
          value={searchQuery}
          onChange={handleSearchChange}
          resultCount={searchQuery ? filteredNodes.length : undefined}
        />
      </div>
      
      <div className="schema-browser-content">
        <div className="schema-tree-container">
          <div className="schema-tree" role="tree" aria-label="OCSF Schema">
            {filteredNodes.length === 0 && searchQuery ? (
              <div className="no-results">
                <p>No results for "{searchQuery}"</p>
              </div>
            ) : (
              filteredNodes.map((node) => (
                <SchemaTreeNode
                  key={node.id}
                  node={node}
                  depth={0}
                  isExpanded={expandedNodes.has(node.id)}
                  isSelected={selectedNode?.id === node.id}
                  searchQuery={searchQuery}
                  expandedNodes={expandedNodes}
                  onToggle={handleToggle}
                  onSelect={handleSelect}
                />
              ))
            )}
          </div>
        </div>
        
        <div className="schema-details-container">
          <AttributeDetails node={selectedNode} />
        </div>
      </div>
      
      {schema.version && (
        <div className="schema-browser-footer">
          <span className="schema-version">OCSF v{schema.version}</span>
        </div>
      )}
    </div>
  );
}

export default SchemaBrowser;
