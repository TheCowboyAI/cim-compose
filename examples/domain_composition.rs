//! Example: Document Ingestion and Processing using GraphComposition
//!
//! This example demonstrates the proper separation between:
//! - cim-domain: Defines what things ARE (Entity, Aggregate)
//! - cim-compose: Defines how things COMBINE (GraphComposition)

use cim_compose::{
    GraphComposition, BaseNodeType, BaseRelationshipType,
    Entity, markers::AggregateMarker,
};
use serde_json::json;

fn main() {
    // Create a document entity using cim-domain types
    let document_entity = Entity::<AggregateMarker>::new();
    let document_id = document_entity.id;

    println!("Created Document Entity: {}", document_id);

    // Compose the document into a graph structure using cim-compose
    let mut document_graph = GraphComposition::aggregate("Document", document_id.to_string())
        .add_node(BaseNodeType::Value, "metadata", json!({
            "title": "CIM Architecture Overview",
            "author": "System Documentation",
            "created": "2025-01-11",
            "format": "markdown",
            "size_bytes": 15420
        }))
        .add_node(BaseNodeType::Value, "content", json!({
            "raw_text": "# CIM Architecture\n\nThe Composable Information Machine...",
            "word_count": 2500,
            "language": "en"
        }))
        .add_node(BaseNodeType::Custom("Extraction".to_string()), "entities", json!({
            "people": ["John Doe", "Jane Smith"],
            "organizations": ["CowboyAI", "NATS.io"],
            "technologies": ["Rust", "Bevy", "Event Sourcing", "CQRS"]
        }))
        .add_node(BaseNodeType::Custom("Embedding".to_string()), "semantic", json!({
            "embedding_model": "text-embedding-3-small",
            "dimensions": 1536,
            "vector": "[0.023, -0.145, 0.892, ...]" // Truncated for example
        }));

    // Add relationships showing document processing pipeline
    document_graph = document_graph
        .add_edge_by_label("root", "metadata", BaseRelationshipType::Contains)
        .add_edge_by_label("root", "content", BaseRelationshipType::Contains)
        .add_edge_by_label("content", "entities", BaseRelationshipType::Custom("ExtractedFrom".to_string()))
        .add_edge_by_label("content", "semantic", BaseRelationshipType::Custom("EmbeddedFrom".to_string()));

    println!("\nDocument Graph Structure:");
    println!("- Root: {} (Document Aggregate)", document_graph.composition_root);
    println!("- Nodes: {} total", document_graph.nodes.len());
    println!("- Edges: {} total", document_graph.edges.len());

    // Demonstrate composition with related documents
    let related_doc = GraphComposition::atomic("RelatedDocument", json!({
        "document_id": "doc-456",
        "title": "Event Sourcing Patterns",
        "similarity_score": 0.87
    }));

    // Create a knowledge graph by composing documents
    match document_graph.parallel(&related_doc) {
        Ok(knowledge_graph) => {
            println!("\nKnowledge Graph Created:");
            println!("- Total nodes: {}", knowledge_graph.nodes.len());
            println!("- Total edges: {}", knowledge_graph.edges.len());
            println!("- Can be used for semantic search and navigation");
        }
        Err(e) => println!("Composition failed: {}", e),
    }

    // Show how this fits into the document processing pipeline
    println!("\nDocument Processing Pipeline:");
    println!("1. Ingest: Create Document entity (cim-domain)");
    println!("2. Parse: Extract metadata and content");
    println!("3. Analyze: Extract entities and generate embeddings");
    println!("4. Compose: Build graph structure (cim-compose)");
    println!("5. Connect: Link to related documents in knowledge graph");

    // Show the separation of concerns:
    println!("\nArchitecture Demonstration:");
    println!("- Entity<T> from cim-domain: Provides identity and lifecycle");
    println!("- GraphComposition from cim-compose: Structures document relationships");
    println!("- Domain modules use both to implement document processing logic");
}
