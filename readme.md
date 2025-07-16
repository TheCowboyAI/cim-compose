# CIM Compose

Graph composition library for the Composable Information Machine.

## Overview

CIM Compose provides the fundamental graph composition capabilities that enable domain modules to be composed into complex graph structures. Based on category theory principles, this library allows uniform operations, type-safe composition, and mathematical transformations of domain concepts represented as graphs.

## Architecture

The CIM architecture follows a clear separation of concerns:

- **cim-domain**: Provides core DDD building blocks (Entity, Aggregate, etc.)
- **cim-compose**: Provides graph composition of those building blocks (this crate)
- **Domain modules**: Pure domain logic (Document, Graph, Person, etc.)

### Key Principle

Domain modules are PURE - they only depend on `cim-domain`. The `cim-compose` library depends on domain modules to provide composition capabilities, not the other way around. This prevents circular dependencies and maintains clean architecture.

## Key Concepts

### GraphComposition

The main type for representing composable graphs. Everything in the system can be represented as a GraphComposition:

```rust
// Create an atomic graph (single node)
let money = GraphComposition::atomic(
    "Money",
    json!({ "amount": 100, "currency": "USD" })
);

// Create a composite graph
let order = GraphComposition::composite("Order")
    .add_node(BaseNodeType::Value, "total", money)
    .add_node(BaseNodeType::Value, "status", "pending")
    .add_edge_by_label("root", "total", BaseRelationshipType::Contains);
```

### Node Types

- `Value`: Simple value nodes
- `EntityReference`: References to domain entities
- `Entity`: Cross-references between graphs
- `Aggregate`: Aggregate root nodes
- `Service`: Service nodes
- `Command`: Command nodes
- `Event`: Event nodes
- `Custom(String)`: Extensible for domain-specific types

### Relationship Types

- `Contains`: Parent-child relationships
- `References`: Reference relationships
- `DependsOn`: Dependency relationships
- `Sequence`: Ordered relationships
- `Parallel`: Concurrent relationships
- `Choice`: Alternative relationships
- `Hierarchy`: Organizational relationships
- `Custom(String)`: Extensible for domain-specific relationships

## Composition Operations

### Sequential Composition

Compose graphs in sequence:

```rust
let validate = GraphComposition::composite("ValidateOrder");
let process = GraphComposition::composite("ProcessPayment");

let workflow = validate.then(&process)?;
```

### Parallel Composition

Compose graphs in parallel:

```rust
let check_inventory = GraphComposition::composite("CheckInventory");
let verify_payment = GraphComposition::composite("VerifyPayment");

let parallel_checks = check_inventory.parallel(&verify_payment)?;
```

### Choice Composition

Compose graphs as alternatives:

```rust
let credit_card = GraphComposition::composite("CreditCardPayment");
let paypal = GraphComposition::composite("PayPalPayment");

let payment_options = credit_card.choice(&paypal)?;
```

## Category Theory Operations

### Functors

Structure-preserving maps between graphs:

```rust
let doubled = graph.fmap(|node| {
    let mut new_node = node.clone();
    // Transform node data
    new_node
});
```

### Morphisms

Transform one graph type to another:

```rust
impl GraphMorphism<NodeType1, RelType1, NodeType2, RelType2> for MyMorphism {
    fn apply(&self, graph: &GraphComposition<NodeType1, RelType1>) 
        -> Result<GraphComposition<NodeType2, RelType2>, CompositionError> {
        // Transform graph
    }
}
```

## Domain Composition

When domain features are enabled, you can compose domain aggregates into graphs:

```rust
#[cfg(feature = "document")]
use cim_compose::domain_compositions::document;

// Convert a Document aggregate to a graph
let doc_graph = document.to_graph();

// Create a document processing pipeline
let pipeline = document::create_processing_pipeline();
```

### Available Domain Compositions

- `document`: Document processing and management
- `agent`: Agent networks and capabilities
- `organization`: Organizational hierarchies
- `workflow`: Process orchestration
- `location`: Geospatial relationships
- `person`: Identity and relationships
- `graph`: Graph-based knowledge structures
- `conceptualspaces`: Semantic spaces

## Usage Examples

### Document Processing Pipeline

```rust
use cim_compose::{GraphComposition, BaseNodeType, BaseRelationshipType};

let pipeline = GraphComposition::composite("DocumentPipeline")
    .add_node(BaseNodeType::Custom("Stage".to_string()), "ingest", json!({
        "type": "Document Ingestion",
        "accepts": ["pdf", "docx", "md", "txt"],
    }))
    .add_node(BaseNodeType::Custom("Stage".to_string()), "extract", json!({
        "type": "Text Extraction",
        "output": "plain text",
    }))
    .add_node(BaseNodeType::Custom("Stage".to_string()), "analyze", json!({
        "type": "NLP Analysis",
        "tasks": ["tokenization", "NER", "sentiment"],
    }))
    .add_edge_by_label("ingest", "extract", BaseRelationshipType::Sequence)
    .add_edge_by_label("extract", "analyze", BaseRelationshipType::Sequence);
```

### Entity Links

Entity links are represented as graphs themselves:

```rust
// Create a dependency link between two entities
let dependency = GraphComposition::composite("DependencyLink")
    .add_node(BaseNodeType::EntityReference, "source", json!({
        "entity_id": workflow_id.to_string(),
        "entity_type": "Workflow"
    }))
    .add_node(BaseNodeType::EntityReference, "target", json!({
        "entity_id": data_model_id.to_string(),
        "entity_type": "DataModel"
    }))
    .add_edge_by_label("source", "target", BaseRelationshipType::DependsOn);
```

### Knowledge Graph Composition

```rust
use cim_compose::domain_compositions::compose_knowledge_graph;

// Compose multiple domain objects into a knowledge graph
let objects = vec![document1, document2, person1];
let knowledge_graph = compose_knowledge_graph(&objects);
```

## Features

Enable domain-specific compositions with feature flags:

```toml
[dependencies]
cim-compose = { version = "0.1", features = ["document", "agent", "workflow"] }

# Or enable all domains
cim-compose = { version = "0.1", features = ["all-domains"] }
```

## Invariants and Validation

Add invariant constraints to graphs:

```rust
let graph = GraphComposition::composite("Order")
    .with_invariant(|g| {
        // Ensure order has at least one line item
        g.nodes.values().any(|n| n.label == "line_item")
    })
    .with_invariant(|g| {
        // Ensure total is non-negative
        true // Implement validation logic
    });

// Check invariants
graph.check_invariants()?;
```

## Graph Analysis

### Find Leaf Nodes

```rust
let leaves = graph.find_leaves();
```

### Find Root Nodes

```rust
let roots = graph.find_roots();
```

### Get Connected Nodes

```rust
let connected = graph.get_connected_nodes(node_id);
```

### Fold Operations

```rust
let total = graph.fold(0.0, |acc, node| {
    // Accumulate values from nodes
    acc + extract_value(node)
});
```

## Error Handling

```rust
use cim_compose::CompositionError;

match graph1.compose(&graph2) {
    Ok(composed) => {
        // Use composed graph
    }
    Err(CompositionError::IncompatibleTypes(t1, t2)) => {
        // Handle type mismatch
    }
    Err(CompositionError::CycleDetected) => {
        // Handle circular dependency
    }
    Err(e) => {
        // Handle other errors
    }
}
```

## Performance Considerations

- Graphs are cloned during composition operations
- Use references when possible for read-only operations
- Invariant functions cannot be serialized
- Consider graph size when using recursive operations

## Testing

```bash
# Run all tests
cargo test -p cim-compose

# Run with all features
cargo test -p cim-compose --all-features

# Run specific test
cargo test -p cim-compose test_sequential_composition
```

## Contributing

1. Maintain separation between domain logic and composition
2. Domain modules should never depend on cim-compose
3. Use feature flags for optional domain integrations
4. Add tests for new composition operations
5. Document mathematical foundations when applicable

## License

See the main project LICENSE file. 