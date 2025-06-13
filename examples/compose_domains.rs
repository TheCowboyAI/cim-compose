//! Example: Composing Domain Modules
//!
//! This example demonstrates the correct architecture:
//! - Domain modules (Document, Graph, Person) are pure domain logic with no composition dependencies
//! - cim-compose depends on domain modules to provide composition capabilities
//! - The composition layer can combine multiple domains into knowledge graphs

// Note: This example requires features to be enabled
// Run with: cargo run --example compose_domains --features all-domains

fn main() {
    println!("=== Domain Composition Architecture ===\n");

    println!("Architecture Overview:");
    println!("1. Domain modules are PURE - they only depend on cim-domain");
    println!("2. cim-compose DEPENDS ON domain modules (not vice versa)");
    println!("3. cim-compose provides composition capabilities for domains");

    #[cfg(feature = "document")]
    {
        use cim_compose::domain_compositions::document::create_processing_pipeline;

        println!("\n=== Document Processing Pipeline ===");
        let pipeline = create_processing_pipeline();
        println!("Created pipeline with {} nodes", pipeline.nodes.len());
        println!("Pipeline stages:");
        for (_, node) in &pipeline.nodes {
            if node.label != "root" {
                println!("  - {}: {}", node.label, node.data["type"]);
            }
        }
    }

    #[cfg(feature = "agent")]
    {
        use cim_compose::domain_compositions::agent::create_agent_network;

        println!("\n=== Agent Network ===");
        let network = create_agent_network();
        println!("Created agent network with {} nodes", network.nodes.len());
        println!("Agent types and capabilities:");
        for (_, node) in &network.nodes {
            if node.label != "root" {
                if let Some(node_type) = node.data.get("type") {
                    println!("  - {}: {}", node.label, node_type);
                } else if let Some(name) = node.data.get("name") {
                    println!("  - {}: {}", node.label, name);
                }
            }
        }
    }

    #[cfg(feature = "organization")]
    {
        use cim_compose::domain_compositions::organization::create_org_hierarchy;

        println!("\n=== Organizational Hierarchy ===");
        let hierarchy = create_org_hierarchy();
        println!("Created org hierarchy with {} levels", hierarchy.nodes.len() - 1);
        println!("Hierarchy levels:");
        for (_, node) in &hierarchy.nodes {
            if node.label != "root" {
                println!("  - {}: {}", node.data["level"], node.data["description"]);
            }
        }
    }

    #[cfg(not(any(feature = "document", feature = "agent", feature = "organization")))]
    {
        println!("\n=== No Features Enabled ===");
        println!("Run with features to see domain compositions:");
        println!("  cargo run --example compose_domains --features document");
        println!("  cargo run --example compose_domains --features agent");
        println!("  cargo run --example compose_domains --features organization");
        println!("  cargo run --example compose_domains --features all-domains");
    }

    println!("\n=== Key Benefits ===");
    println!("1. Domain modules remain pure and focused on business logic");
    println!("2. Composition logic is centralized in cim-compose");
    println!("3. New domains can be added without modifying existing ones");
    println!("4. Feature flags allow selective compilation");
    println!("5. Clear dependency direction prevents circular dependencies");

    #[cfg(feature = "conceptualspaces")]
    {
        use cim_compose::domain_compositions::conceptualspaces::create_conceptual_space_viz;

        println!("\n=== Conceptual Space Visualization ===");
        let viz = create_conceptual_space_viz();
        println!("Created conceptual space visualization with {} components", viz.nodes.len() - 1);
        println!("Components:");
        for (_, node) in &viz.nodes {
            if node.label != "root" {
                println!("  - {}: {}", node.data["type"], node.data["description"]);
            }
        }
    }

    println!("\n=== Domain Count ===");
    let mut enabled_domains = Vec::new();
    #[cfg(feature = "document")] enabled_domains.push("Document");
    #[cfg(feature = "graph")] enabled_domains.push("Graph");
    #[cfg(feature = "person")] enabled_domains.push("Person");
    #[cfg(feature = "workflow")] enabled_domains.push("Workflow");
    #[cfg(feature = "location")] enabled_domains.push("Location");
    #[cfg(feature = "agent")] enabled_domains.push("Agent");
    #[cfg(feature = "organization")] enabled_domains.push("Organization");
    #[cfg(feature = "conceptualspaces")] enabled_domains.push("ConceptualSpaces");

    println!("Enabled domains: {} of 8", enabled_domains.len());
    if !enabled_domains.is_empty() {
        println!("  - {}", enabled_domains.join("\n  - "));
    }
}
