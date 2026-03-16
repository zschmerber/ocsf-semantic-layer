//! Synonym resolution for natural language queries.
//!
//! This module provides functionality to resolve synonyms and alternative
//! field names to their canonical attribute names in the semantic model.

use std::collections::HashMap;
use crate::entity::SemanticAttribute;
use crate::model::SemanticModel;

/// A synonym resolver that maps alternative names to canonical field names.
#[derive(Debug, Clone, Default)]
pub struct SynonymResolver {
    /// Map from lowercase synonym to (entity_name, canonical_attribute_name).
    synonym_index: HashMap<String, (String, String)>,
    /// Map from lowercase canonical name to (entity_name, canonical_attribute_name).
    canonical_index: HashMap<String, (String, String)>,
}

impl SynonymResolver {
    /// Creates a new empty synonym resolver.
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds a synonym resolver from a semantic model.
    pub fn from_model(model: &SemanticModel) -> Self {
        let mut resolver = Self::new();
        
        for entity in &model.entities {
            for attr in &entity.attributes {
                resolver.add_attribute(&entity.name, attr);
            }
        }
        
        resolver
    }

    /// Adds an attribute and its synonyms to the resolver.
    pub fn add_attribute(&mut self, entity_name: &str, attr: &SemanticAttribute) {
        let canonical_key = attr.name.to_lowercase();
        let entry = (entity_name.to_string(), attr.name.clone());
        
        // Add canonical name
        self.canonical_index.insert(canonical_key.clone(), entry.clone());
        
        // Add synonyms
        for synonym in &attr.synonyms {
            let synonym_key = synonym.to_lowercase();
            self.synonym_index.insert(synonym_key, entry.clone());
        }
    }

    /// Resolves a term to its canonical attribute name.
    ///
    /// Returns `Some((entity_name, canonical_name))` if found, `None` otherwise.
    /// Matching is case-insensitive.
    pub fn resolve(&self, term: &str) -> Option<(&str, &str)> {
        let key = term.to_lowercase();
        
        // First check canonical names
        if let Some((entity, attr)) = self.canonical_index.get(&key) {
            return Some((entity.as_str(), attr.as_str()));
        }
        
        // Then check synonyms
        if let Some((entity, attr)) = self.synonym_index.get(&key) {
            return Some((entity.as_str(), attr.as_str()));
        }
        
        None
    }

    /// Resolves a term to just the canonical attribute name.
    pub fn resolve_attribute(&self, term: &str) -> Option<&str> {
        self.resolve(term).map(|(_, attr)| attr)
    }

    /// Resolves a term to just the entity name.
    pub fn resolve_entity(&self, term: &str) -> Option<&str> {
        self.resolve(term).map(|(entity, _)| entity)
    }

    /// Returns all synonyms for a given canonical attribute name.
    pub fn get_synonyms(&self, canonical_name: &str) -> Vec<&str> {
        let key = canonical_name.to_lowercase();
        
        self.synonym_index
            .iter()
            .filter(|(_, (_, attr))| attr.to_lowercase() == key)
            .map(|(synonym, _)| synonym.as_str())
            .collect()
    }

    /// Returns all registered canonical attribute names.
    pub fn canonical_names(&self) -> impl Iterator<Item = &str> {
        self.canonical_index.values().map(|(_, attr)| attr.as_str())
    }

    /// Returns the total number of synonyms registered.
    pub fn synonym_count(&self) -> usize {
        self.synonym_index.len()
    }

    /// Returns the total number of canonical names registered.
    pub fn canonical_count(&self) -> usize {
        self.canonical_index.len()
    }

    /// Checks if a term is a known synonym or canonical name.
    pub fn contains(&self, term: &str) -> bool {
        self.resolve(term).is_some()
    }

    /// Suggests possible matches for a term using fuzzy matching.
    ///
    /// Returns up to `max_suggestions` terms that are similar to the input.
    pub fn suggest(&self, term: &str, max_suggestions: usize) -> Vec<&str> {
        let term_lower = term.to_lowercase();
        let mut suggestions: Vec<(&str, usize)> = Vec::new();
        
        // Check canonical names
        for (key, (_, attr)) in &self.canonical_index {
            if let Some(score) = Self::similarity_score(&term_lower, key) {
                suggestions.push((attr.as_str(), score));
            }
        }
        
        // Check synonyms
        for (synonym, (_, attr)) in &self.synonym_index {
            if let Some(score) = Self::similarity_score(&term_lower, synonym) {
                suggestions.push((attr.as_str(), score));
            }
        }
        
        // Sort by score (higher is better) and deduplicate
        suggestions.sort_by(|a, b| b.1.cmp(&a.1));
        
        let mut seen = std::collections::HashSet::new();
        suggestions
            .into_iter()
            .filter(|(attr, _)| seen.insert(*attr))
            .take(max_suggestions)
            .map(|(attr, _)| attr)
            .collect()
    }

    /// Calculates a simple similarity score between two strings.
    /// Returns None if strings are too dissimilar.
    fn similarity_score(a: &str, b: &str) -> Option<usize> {
        // Simple substring matching
        if a.contains(b) || b.contains(a) {
            return Some(100);
        }
        
        // Check for common prefix
        let common_prefix = a.chars()
            .zip(b.chars())
            .take_while(|(ca, cb)| ca == cb)
            .count();
        
        if common_prefix >= 3 {
            return Some(common_prefix * 10);
        }
        
        // Check for word overlap
        let a_words: std::collections::HashSet<_> = a.split_whitespace().collect();
        let b_words: std::collections::HashSet<_> = b.split_whitespace().collect();
        let overlap = a_words.intersection(&b_words).count();
        
        if overlap > 0 {
            return Some(overlap * 20);
        }
        
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{SemanticEntity, SemanticAttribute};

    fn create_test_model() -> SemanticModel {
        SemanticModel::new("test")
            .add_entity(
                SemanticEntity::new("dns_event")
                    .add_attribute(
                        SemanticAttribute::new("query_hostname")
                            .with_synonyms(vec![
                                "domain".to_string(),
                                "dns name".to_string(),
                                "fqdn".to_string(),
                            ])
                    )
                    .add_attribute(
                        SemanticAttribute::new("source_ip")
                            .with_synonyms(vec![
                                "client IP".to_string(),
                                "origin IP".to_string(),
                                "src IP".to_string(),
                            ])
                    )
            )
    }

    #[test]
    fn test_resolve_canonical_name() {
        let model = create_test_model();
        let resolver = SynonymResolver::from_model(&model);

        let result = resolver.resolve("query_hostname");
        assert!(result.is_some());
        let (entity, attr) = result.unwrap();
        assert_eq!(entity, "dns_event");
        assert_eq!(attr, "query_hostname");
    }

    #[test]
    fn test_resolve_synonym() {
        let model = create_test_model();
        let resolver = SynonymResolver::from_model(&model);

        let result = resolver.resolve("domain");
        assert!(result.is_some());
        let (entity, attr) = result.unwrap();
        assert_eq!(entity, "dns_event");
        assert_eq!(attr, "query_hostname");
    }

    #[test]
    fn test_case_insensitive() {
        let model = create_test_model();
        let resolver = SynonymResolver::from_model(&model);

        assert!(resolver.resolve("DOMAIN").is_some());
        assert!(resolver.resolve("Domain").is_some());
        assert!(resolver.resolve("QUERY_HOSTNAME").is_some());
    }

    #[test]
    fn test_resolve_unknown() {
        let model = create_test_model();
        let resolver = SynonymResolver::from_model(&model);

        assert!(resolver.resolve("unknown_field").is_none());
    }

    #[test]
    fn test_get_synonyms() {
        let model = create_test_model();
        let resolver = SynonymResolver::from_model(&model);

        let synonyms = resolver.get_synonyms("query_hostname");
        assert_eq!(synonyms.len(), 3);
        assert!(synonyms.contains(&"domain"));
        assert!(synonyms.contains(&"dns name"));
        assert!(synonyms.contains(&"fqdn"));
    }

    #[test]
    fn test_counts() {
        let model = create_test_model();
        let resolver = SynonymResolver::from_model(&model);

        assert_eq!(resolver.canonical_count(), 2);
        assert_eq!(resolver.synonym_count(), 6);
    }

    #[test]
    fn test_contains() {
        let model = create_test_model();
        let resolver = SynonymResolver::from_model(&model);

        assert!(resolver.contains("query_hostname"));
        assert!(resolver.contains("domain"));
        assert!(resolver.contains("client IP"));
        assert!(!resolver.contains("unknown"));
    }

    #[test]
    fn test_suggest() {
        let model = create_test_model();
        let resolver = SynonymResolver::from_model(&model);

        let suggestions = resolver.suggest("dom", 5);
        assert!(!suggestions.is_empty());
        // "domain" should match and resolve to "query_hostname"
    }

    #[test]
    fn test_resolve_attribute() {
        let model = create_test_model();
        let resolver = SynonymResolver::from_model(&model);

        assert_eq!(resolver.resolve_attribute("domain"), Some("query_hostname"));
        assert_eq!(resolver.resolve_attribute("client IP"), Some("source_ip"));
    }
}
