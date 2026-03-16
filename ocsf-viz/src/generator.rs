//! Graph generation from semantic models and OCSF schemas.
//!
//! This module provides functionality to generate visualization graphs
//! from semantic models and OCSF schemas.

use std::collections::HashSet;

use ocsf_core::OCSFSchema;
use ocsf_semantic::{SemanticEntity, SemanticModel};

use crate::graph::{
    EdgeType, GraphData, GraphEdge, GraphMetadata, GraphNode, LayoutType, NodeType, ViewMode,
};

/// Options for graph generation.
#[derive(Debug, Clone)]
pub struct GeneratorOptions {
    /// View mode for the generated graph.
    pub view_mode: ViewMode,
    /// Whether to include observables in the graph.
    pub show_observables: bool,
    /// Whether to include entity relationships.
    pub show_relationships: bool,
    /// Layout algorithm to use.
    pub layout: LayoutType,
    /// Whether to include OCSF attributes (can make graph very large).
    pub include_attributes: bool,
}

impl Default for GeneratorOptions {
    fn default() -> Self {
        Self {
            view_mode: ViewMode::Interchange,
            show_observables: true,
            show_relationships: true,
            layout: LayoutType::Hierarchical,
            include_attributes: false,
        }
    }
}

impl GeneratorOptions {
    /// Creates new options with the given view mode.
    pub fn with_view_mode(view_mode: ViewMode) -> Self {
        Self {
            view_mode,
            ..Default::default()
        }
    }

    /// Sets whether to show observables.
    pub fn show_observables(mut self, show: bool) -> Self {
        self.show_observables = show;
        self
    }

    /// Sets whether to show relationships.
    pub fn show_relationships(mut self, show: bool) -> Self {
        self.show_relationships = show;
        self
    }

    /// Sets the layout type.
    pub fn with_layout(mut self, layout: LayoutType) -> Self {
        self.layout = layout;
        self
    }

    /// Sets whether to include attributes.
    pub fn include_attributes(mut self, include: bool) -> Self {
        self.include_attributes = include;
        self
    }
}

/// Generator for creating visualization graphs.
#[derive(Debug)]
pub struct GraphGenerator<'a> {
    /// The semantic model.
    model: &'a SemanticModel,
    /// The OCSF schema.
    schema: &'a OCSFSchema,
    /// Generation options.
    options: GeneratorOptions,
}

impl<'a> GraphGenerator<'a> {
    /// Creates a new graph generator.
    pub fn new(model: &'a SemanticModel, schema: &'a OCSFSchema) -> Self {
        Self {
            model,
            schema,
            options: GeneratorOptions::default(),
        }
    }

    /// Creates a new graph generator with options.
    pub fn with_options(
        model: &'a SemanticModel,
        schema: &'a OCSFSchema,
        options: GeneratorOptions,
    ) -> Self {
        Self {
            model,
            schema,
            options,
        }
    }

    /// Generates the visualization graph.
    pub fn generate(&self) -> GraphData {
        let mut graph = GraphData::with_view_mode(self.options.view_mode);
        graph.metadata = self.create_metadata();

        // Generate nodes and edges based on view mode
        match self.options.view_mode {
            ViewMode::SemanticOnly => {
                self.add_semantic_nodes(&mut graph);
                if self.options.show_relationships {
                    self.add_relationship_edges(&mut graph);
                }
            }
            ViewMode::PhysicalOnly => {
                self.add_physical_nodes(&mut graph);
            }
            ViewMode::Interchange => {
                self.add_semantic_nodes(&mut graph);
                self.add_physical_nodes(&mut graph);
                self.add_mapping_edges(&mut graph);
                if self.options.show_relationships {
                    self.add_relationship_edges(&mut graph);
                }
                if self.options.show_observables {
                    self.add_observable_nodes(&mut graph);
                    self.add_observable_coverage_edges(&mut graph);
                }
            }
            ViewMode::HotColdPath => {
                self.add_hot_cold_path_nodes(&mut graph);
                if self.options.show_observables {
                    self.add_observable_nodes(&mut graph);
                }
            }
        }

        graph
    }

    /// Creates metadata for the graph.
    fn create_metadata(&self) -> GraphMetadata {
        GraphMetadata::new(self.options.view_mode)
            .with_schema_version(&self.schema.version)
            .with_model_version(&self.model.version)
            .with_layout(self.options.layout)
    }

    /// Adds semantic entity nodes to the graph.
    fn add_semantic_nodes(&self, graph: &mut GraphData) {
        for entity in &self.model.entities {
            let node = self.create_entity_node(entity);
            graph.add_node(node);
        }
    }

    /// Creates a node for a semantic entity.
    fn create_entity_node(&self, entity: &SemanticEntity) -> GraphNode {
        GraphNode::new(
            format!("entity_{}", entity.name),
            NodeType::SemanticEntity,
            &entity.caption,
        )
        .with_property("name", entity.name.clone())
        .with_property("description", entity.description.clone())
        .with_property(
            "source_event_classes",
            serde_json::json!(entity.source_event_classes),
        )
        .with_property("attribute_count", entity.attributes.len())
    }

    /// Adds physical OCSF nodes to the graph.
    fn add_physical_nodes(&self, graph: &mut GraphData) {
        // Track which categories and classes are referenced by entities
        let referenced_classes: HashSet<u32> = self
            .model
            .entities
            .iter()
            .flat_map(|e| e.source_event_classes.iter().copied())
            .collect();

        let referenced_categories: HashSet<u32> = referenced_classes
            .iter()
            .filter_map(|&class_uid| {
                self.schema
                    .get_event_class(class_uid)
                    .map(|ec| ec.category_uid)
            })
            .collect();

        // Add category nodes
        for category in self.schema.all_categories() {
            // Only add categories that are referenced or if we're showing all
            if referenced_categories.contains(&category.uid) || referenced_categories.is_empty() {
                let node = GraphNode::new(
                    format!("category_{}", category.uid),
                    NodeType::OcsfCategory,
                    &category.caption,
                )
                .with_property("uid", category.uid)
                .with_property("name", category.name.clone())
                .with_property("description", category.description.clone());
                graph.add_node(node);
            }
        }

        // Add event class nodes
        for event_class in self.schema.all_event_classes() {
            // Only add classes that are referenced or if we're showing all
            if referenced_classes.contains(&event_class.class_uid) || referenced_classes.is_empty()
            {
                let node = GraphNode::new(
                    format!("class_{}", event_class.class_uid),
                    NodeType::OcsfClass,
                    &event_class.caption,
                )
                .with_property("class_uid", event_class.class_uid)
                .with_property("category_uid", event_class.category_uid)
                .with_property("name", event_class.name.clone())
                .with_property("description", event_class.description.clone());
                graph.add_node(node);

                // Add contains edge from category to class
                let category_id = format!("category_{}", event_class.category_uid);
                if graph.get_node(&category_id).is_some() {
                    graph.add_edge(GraphEdge::new(
                        category_id,
                        format!("class_{}", event_class.class_uid),
                        EdgeType::Contains,
                    ));
                }

                // Optionally add attribute nodes
                if self.options.include_attributes {
                    for attr_name in event_class.attributes.keys() {
                        let attr_id = format!("attr_{}_{}", event_class.class_uid, attr_name);
                        let attr_node = GraphNode::new(&attr_id, NodeType::OcsfAttribute, attr_name)
                            .with_property("class_uid", event_class.class_uid)
                            .with_property("name", attr_name.clone());
                        graph.add_node(attr_node);

                        graph.add_edge(GraphEdge::new(
                            format!("class_{}", event_class.class_uid),
                            attr_id,
                            EdgeType::Contains,
                        ));
                    }
                }
            }
        }
    }

    /// Adds mapping edges between semantic entities and OCSF classes.
    fn add_mapping_edges(&self, graph: &mut GraphData) {
        for entity in &self.model.entities {
            let entity_id = format!("entity_{}", entity.name);

            for &class_uid in &entity.source_event_classes {
                let class_id = format!("class_{}", class_uid);

                // Only add edge if both nodes exist
                if graph.get_node(&entity_id).is_some() && graph.get_node(&class_id).is_some() {
                    graph.add_edge(GraphEdge::new(&entity_id, &class_id, EdgeType::MapsTo));
                }
            }
        }
    }

    /// Adds relationship edges between semantic entities.
    fn add_relationship_edges(&self, graph: &mut GraphData) {
        for entity in &self.model.entities {
            let source_id = format!("entity_{}", entity.name);

            for relationship in &entity.relationships {
                let target_id = format!("entity_{}", relationship.target_entity);

                // Only add edge if both nodes exist
                if graph.get_node(&source_id).is_some() && graph.get_node(&target_id).is_some() {
                    graph.add_edge(
                        GraphEdge::new(&source_id, &target_id, EdgeType::RelatesTo)
                            .with_label(&relationship.name),
                    );
                }
            }
        }
    }

    /// Adds observable nodes to the graph.
    fn add_observable_nodes(&self, graph: &mut GraphData) {
        // Get unique observable type_ids from the schema
        let mut added_observables: HashSet<u32> = HashSet::new();

        for observable in self.schema.all_observables() {
            if !added_observables.contains(&observable.type_id) {
                let node = GraphNode::new(
                    format!("observable_{}", observable.type_id),
                    NodeType::Observable,
                    &observable.type_name,
                )
                .with_property("type_id", observable.type_id)
                .with_property("type_name", observable.type_name.clone())
                .with_property("description", observable.description.clone());
                graph.add_node(node);
                added_observables.insert(observable.type_id);
            }
        }
    }

    /// Adds edges showing which entities cover which observables.
    fn add_observable_coverage_edges(&self, graph: &mut GraphData) {
        for entity in &self.model.entities {
            let entity_id = format!("entity_{}", entity.name);

            for &type_id in &entity.covers_observables {
                let observable_id = format!("observable_{}", type_id);

                // Only add edge if both nodes exist
                if graph.get_node(&entity_id).is_some() && graph.get_node(&observable_id).is_some()
                {
                    graph.add_edge(GraphEdge::new(
                        &entity_id,
                        &observable_id,
                        EdgeType::CoversObservable,
                    ));
                }
            }
        }
    }

    /// Adds hot/cold path nodes for data flow visualization.
    fn add_hot_cold_path_nodes(&self, graph: &mut GraphData) {
        // Add hot path node (observables table)
        let hot_path_node = GraphNode::new("hot_path", NodeType::HotPath, "Observables Table")
            .with_property("description", "Denormalized observables for fast threat intel matching")
            .with_property("table_name", self.model.observable_config.table_name.clone());
        graph.add_node(hot_path_node);

        // Add cold path node (full events table)
        let cold_path_node = GraphNode::new("cold_path", NodeType::ColdPath, "Full Events Table")
            .with_property("description", "Complete OCSF event data for detailed investigation");
        graph.add_node(cold_path_node);

        // Add reverse lookup edge
        graph.add_edge(
            GraphEdge::new("hot_path", "cold_path", EdgeType::ReverseLookup)
                .with_label("reverse lookup"),
        );

        // Add data flow edges from observables to hot path
        for observable in self.schema.all_observables() {
            let observable_id = format!("observable_{}", observable.type_id);
            if graph.get_node(&observable_id).is_some() {
                graph.add_edge(GraphEdge::new(&observable_id, "hot_path", EdgeType::DataFlow));
            }
        }
    }
}

/// Generates a visualization graph from a semantic model and OCSF schema.
///
/// This is a convenience function that creates a generator and generates the graph.
pub fn generate_graph(
    model: &SemanticModel,
    schema: &OCSFSchema,
    options: GeneratorOptions,
) -> GraphData {
    GraphGenerator::with_options(model, schema, options).generate()
}

/// Generates a graph showing only semantic entities and their relationships.
pub fn generate_semantic_graph(model: &SemanticModel, schema: &OCSFSchema) -> GraphData {
    generate_graph(
        model,
        schema,
        GeneratorOptions::with_view_mode(ViewMode::SemanticOnly),
    )
}

/// Generates a graph showing only physical OCSF elements.
pub fn generate_physical_graph(model: &SemanticModel, schema: &OCSFSchema) -> GraphData {
    generate_graph(
        model,
        schema,
        GeneratorOptions::with_view_mode(ViewMode::PhysicalOnly),
    )
}

/// Generates a full interchange graph showing both layers.
pub fn generate_interchange_graph(model: &SemanticModel, schema: &OCSFSchema) -> GraphData {
    generate_graph(
        model,
        schema,
        GeneratorOptions::with_view_mode(ViewMode::Interchange),
    )
}

/// Generates a hot/cold path visualization graph.
pub fn generate_hot_cold_path_graph(model: &SemanticModel, schema: &OCSFSchema) -> GraphData {
    generate_graph(
        model,
        schema,
        GeneratorOptions::with_view_mode(ViewMode::HotColdPath),
    )
}


#[cfg(test)]
mod tests {
    use super::*;
    use ocsf_core::{Category, EventClass, ObservableDefinition};
    use ocsf_semantic::{
        Cardinality, EntityRelationship, ObservableConfig, SemanticAttribute, SemanticEntity,
    };
    use std::collections::HashMap;

    fn create_test_schema() -> OCSFSchema {
        let mut schema = OCSFSchema::new("1.4.0");

        // Add a category
        schema.add_category(Category {
            uid: 3,
            name: "iam".to_string(),
            caption: "Identity & Access Management".to_string(),
            description: "IAM events".to_string(),
            event_classes: vec![],
        });

        // Add event classes
        schema.add_event_class(EventClass {
            class_uid: 3002,
            category_uid: 3,
            name: "authentication".to_string(),
            caption: "Authentication".to_string(),
            description: "Authentication events".to_string(),
            attributes: HashMap::new(),
            observables: vec![],
            extends: None,
            profiles: vec![],
        });

        schema.add_event_class(EventClass {
            class_uid: 3003,
            category_uid: 3,
            name: "account_change".to_string(),
            caption: "Account Change".to_string(),
            description: "Account change events".to_string(),
            attributes: HashMap::new(),
            observables: vec![],
            extends: None,
            profiles: vec![],
        });

        // Add observables
        schema.add_observable(
            ObservableDefinition::by_type(2, "IP Address")
                .with_description("IP address observable"),
        );
        schema.add_observable(
            ObservableDefinition::by_type(5, "Email Address")
                .with_description("Email address observable"),
        );
        schema.add_observable(
            ObservableDefinition::by_type(10, "User Name")
                .with_description("User name observable"),
        );

        schema
    }

    fn create_test_model() -> SemanticModel {
        SemanticModel::new("test-model")
            .with_version("1.0")
            .with_ocsf_version("1.4.0")
            .add_entity(
                SemanticEntity::new("authentication_event")
                    .with_caption("Authentication Event")
                    .with_description("User authentication attempts")
                    .with_source_event_classes(vec![3002, 3003])
                    .with_covers_observables(vec![5, 10])
                    .add_attribute(
                        SemanticAttribute::new("user_email")
                            .with_field_mapping("actor.user.email_addr"),
                    )
                    .add_relationship(
                        EntityRelationship::new(
                            "performed_by",
                            "user",
                            "authentication_event.user_email = user.email",
                        )
                        .with_cardinality(Cardinality::ManyToMany),
                    ),
            )
            .add_entity(
                SemanticEntity::new("user")
                    .with_caption("User")
                    .with_description("User entity")
                    .with_source_event_classes(vec![3002])
                    .with_covers_observables(vec![10])
                    .add_attribute(
                        SemanticAttribute::new("email").with_field_mapping("user.email_addr"),
                    ),
            )
            .with_observable_config(ObservableConfig {
                extract_to_table: true,
                table_name: "ocsf_observables".to_string(),
                include_types: vec![2, 5, 10],
            })
    }

    #[test]
    fn test_generate_semantic_only_graph() {
        let schema = create_test_schema();
        let model = create_test_model();

        let graph = generate_semantic_graph(&model, &schema);

        // Should have 2 entity nodes
        assert_eq!(graph.nodes_of_type(NodeType::SemanticEntity).count(), 2);

        // Should have no physical nodes
        assert_eq!(graph.nodes_of_type(NodeType::OcsfCategory).count(), 0);
        assert_eq!(graph.nodes_of_type(NodeType::OcsfClass).count(), 0);

        // Should have relationship edge
        let relates_to_edges: Vec<_> = graph.edges_of_type(EdgeType::RelatesTo).collect();
        assert_eq!(relates_to_edges.len(), 1);
        assert_eq!(relates_to_edges[0].label, Some("performed_by".to_string()));
    }

    #[test]
    fn test_generate_physical_only_graph() {
        let schema = create_test_schema();
        let model = create_test_model();

        let graph = generate_physical_graph(&model, &schema);

        // Should have category and class nodes
        assert!(graph.nodes_of_type(NodeType::OcsfCategory).count() > 0);
        assert!(graph.nodes_of_type(NodeType::OcsfClass).count() > 0);

        // Should have no semantic nodes
        assert_eq!(graph.nodes_of_type(NodeType::SemanticEntity).count(), 0);

        // Should have contains edges
        assert!(graph.edges_of_type(EdgeType::Contains).count() > 0);
    }

    #[test]
    fn test_generate_interchange_graph() {
        let schema = create_test_schema();
        let model = create_test_model();

        let graph = generate_interchange_graph(&model, &schema);

        // Should have both semantic and physical nodes
        assert!(graph.nodes_of_type(NodeType::SemanticEntity).count() > 0);
        assert!(graph.nodes_of_type(NodeType::OcsfCategory).count() > 0);
        assert!(graph.nodes_of_type(NodeType::OcsfClass).count() > 0);

        // Should have mapping edges
        assert!(graph.edges_of_type(EdgeType::MapsTo).count() > 0);

        // Should have observable nodes
        assert!(graph.nodes_of_type(NodeType::Observable).count() > 0);

        // Should have observable coverage edges
        assert!(graph.edges_of_type(EdgeType::CoversObservable).count() > 0);
    }

    #[test]
    fn test_generate_hot_cold_path_graph() {
        let schema = create_test_schema();
        let model = create_test_model();

        let graph = generate_hot_cold_path_graph(&model, &schema);

        // Should have hot and cold path nodes
        assert_eq!(graph.nodes_of_type(NodeType::HotPath).count(), 1);
        assert_eq!(graph.nodes_of_type(NodeType::ColdPath).count(), 1);

        // Should have reverse lookup edge
        let reverse_lookup_edges: Vec<_> = graph.edges_of_type(EdgeType::ReverseLookup).collect();
        assert_eq!(reverse_lookup_edges.len(), 1);

        // Should have observable nodes
        assert!(graph.nodes_of_type(NodeType::Observable).count() > 0);
    }

    #[test]
    fn test_entity_node_properties() {
        let schema = create_test_schema();
        let model = create_test_model();

        let graph = generate_semantic_graph(&model, &schema);

        let auth_node = graph.get_node("entity_authentication_event").unwrap();
        assert_eq!(auth_node.label, "Authentication Event");
        assert_eq!(
            auth_node.get_property("name").unwrap(),
            &serde_json::json!("authentication_event")
        );
        assert_eq!(
            auth_node.get_property("source_event_classes").unwrap(),
            &serde_json::json!([3002, 3003])
        );
    }

    #[test]
    fn test_mapping_edges_connect_entities_to_classes() {
        let schema = create_test_schema();
        let model = create_test_model();

        let graph = generate_interchange_graph(&model, &schema);

        // Find mapping edges from authentication_event
        let auth_edges: Vec<_> = graph
            .edges_of_type(EdgeType::MapsTo)
            .filter(|e| e.source == "entity_authentication_event")
            .collect();

        // Should map to both 3002 and 3003
        assert_eq!(auth_edges.len(), 2);
        let targets: Vec<_> = auth_edges.iter().map(|e| e.target.as_str()).collect();
        assert!(targets.contains(&"class_3002"));
        assert!(targets.contains(&"class_3003"));
    }

    #[test]
    fn test_observable_coverage_edges() {
        let schema = create_test_schema();
        let model = create_test_model();

        let graph = generate_interchange_graph(&model, &schema);

        // authentication_event covers observables 5 and 10
        let auth_coverage: Vec<_> = graph
            .edges_of_type(EdgeType::CoversObservable)
            .filter(|e| e.source == "entity_authentication_event")
            .collect();

        assert_eq!(auth_coverage.len(), 2);
    }

    #[test]
    fn test_graph_metadata() {
        let schema = create_test_schema();
        let model = create_test_model();

        let graph = generate_interchange_graph(&model, &schema);

        assert_eq!(graph.metadata.view_mode, ViewMode::Interchange);
        assert_eq!(graph.metadata.schema_version, Some("1.4.0".to_string()));
        assert_eq!(graph.metadata.model_version, Some("1.0".to_string()));
    }

    #[test]
    fn test_generator_options() {
        let schema = create_test_schema();
        let model = create_test_model();

        // Test with observables disabled
        let options = GeneratorOptions::with_view_mode(ViewMode::Interchange)
            .show_observables(false)
            .show_relationships(false);

        let graph = generate_graph(&model, &schema, options);

        // Should have no observable nodes
        assert_eq!(graph.nodes_of_type(NodeType::Observable).count(), 0);

        // Should have no relationship edges
        assert_eq!(graph.edges_of_type(EdgeType::RelatesTo).count(), 0);
    }

    #[test]
    fn test_empty_model() {
        let schema = create_test_schema();
        let model = SemanticModel::new("empty");

        let graph = generate_interchange_graph(&model, &schema);

        // Should have no semantic nodes
        assert_eq!(graph.nodes_of_type(NodeType::SemanticEntity).count(), 0);

        // Should still have observable nodes from schema
        assert!(graph.nodes_of_type(NodeType::Observable).count() > 0);
    }

    #[test]
    fn test_empty_schema() {
        let schema = OCSFSchema::new("1.0.0");
        let model = create_test_model();

        let graph = generate_interchange_graph(&model, &schema);

        // Should have semantic nodes
        assert!(graph.nodes_of_type(NodeType::SemanticEntity).count() > 0);

        // Should have no physical nodes
        assert_eq!(graph.nodes_of_type(NodeType::OcsfCategory).count(), 0);
        assert_eq!(graph.nodes_of_type(NodeType::OcsfClass).count(), 0);
    }

    #[test]
    fn test_graph_completeness_for_entities() {
        let schema = create_test_schema();
        let model = create_test_model();

        let graph = generate_interchange_graph(&model, &schema);

        // Every entity in the model should have a corresponding node
        for entity in &model.entities {
            let node_id = format!("entity_{}", entity.name);
            assert!(
                graph.get_node(&node_id).is_some(),
                "Missing node for entity: {}",
                entity.name
            );
        }

        // Every source_event_class should have a mapping edge
        for entity in &model.entities {
            let entity_id = format!("entity_{}", entity.name);
            for &class_uid in &entity.source_event_classes {
                let has_edge = graph
                    .edges_of_type(EdgeType::MapsTo)
                    .any(|e| e.source == entity_id && e.target == format!("class_{}", class_uid));
                assert!(
                    has_edge,
                    "Missing mapping edge from {} to class_{}",
                    entity.name, class_uid
                );
            }
        }
    }
}
