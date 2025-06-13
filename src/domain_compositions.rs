//! Domain-specific composition capabilities
//!
//! This module provides traits and implementations for composing
//! domain aggregates from various domain modules into graph structures.

use crate::{GraphComposition, BaseNodeType, BaseRelationshipType, CompositionError, NodeId};
use serde_json::json;

/// Trait for types that can be composed into a GraphComposition
pub trait Composable {
    /// Convert this domain object into a GraphComposition
    fn to_graph(&self) -> GraphComposition;
}

/// Trait for types that can be composed from a GraphComposition
pub trait Decomposable: Sized {
    /// Try to reconstruct this domain object from a GraphComposition
    fn from_graph(graph: &GraphComposition) -> Result<Self, CompositionError>;
}

// Document domain compositions (when feature enabled)
#[cfg(feature = "document")]
pub mod document {
    use super::*;
    use cim_domain::AggregateRoot;
    use cim_domain_document::aggregate::{
        Document, DocumentInfoComponent, ContentAddressComponent,
        ClassificationComponent, LifecycleComponent,
    };

    impl Composable for Document {
        fn to_graph(&self) -> GraphComposition {
            let mut graph = GraphComposition::aggregate("Document", self.id().to_string());

            // Add document info if available
            if let Some(info) = self.get_component::<DocumentInfoComponent>() {
                graph = graph.add_node(
                    BaseNodeType::Value,
                    "info",
                    json!({
                        "title": info.title,
                        "description": info.description,
                        "mime_type": info.mime_type,
                        "filename": info.filename,
                        "size_bytes": info.size_bytes,
                        "language": info.language,
                    })
                );
                graph = graph.add_edge_by_label("root", "info", BaseRelationshipType::Contains);
            }

            // Add content addressing info
            if let Some(content) = self.get_component::<ContentAddressComponent>() {
                graph = graph.add_node(
                    BaseNodeType::Custom("CID".to_string()),
                    "content",
                    json!({
                        "content_cid": content.content_cid.to_string(),
                        "metadata_cid": content.metadata_cid.map(|c| c.to_string()),
                        "hash_algorithm": content.hash_algorithm,
                        "encoding": content.encoding,
                        "is_chunked": content.is_chunked,
                        "chunk_count": content.chunk_cids.len(),
                    })
                );
                graph = graph.add_edge_by_label("root", "content", BaseRelationshipType::Contains);
            }

            // Add classification if available
            if let Some(classification) = self.get_component::<ClassificationComponent>() {
                graph = graph.add_node(
                    BaseNodeType::Value,
                    "classification",
                    json!({
                        "document_type": classification.document_type,
                        "category": classification.category,
                        "subcategories": classification.subcategories,
                        "tags": classification.tags,
                        "confidentiality": format!("{:?}", classification.confidentiality),
                    })
                );
                graph = graph.add_edge_by_label("root", "classification", BaseRelationshipType::Contains);
            }

            // Add lifecycle info if available
            if let Some(lifecycle) = self.get_component::<LifecycleComponent>() {
                graph = graph.add_node(
                    BaseNodeType::Value,
                    "lifecycle",
                    json!({
                        "status": format!("{:?}", lifecycle.status),
                        "created_at": lifecycle.created_at.to_rfc3339(),
                        "modified_at": lifecycle.modified_at.to_rfc3339(),
                        "version_number": lifecycle.version_number,
                    })
                );
                graph = graph.add_edge_by_label("root", "lifecycle", BaseRelationshipType::Contains);
            }

            graph
        }
    }

    /// Create a document processing pipeline graph
    pub fn create_processing_pipeline() -> GraphComposition {
        GraphComposition::composite("DocumentPipeline")
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
            .add_node(BaseNodeType::Custom("Stage".to_string()), "embed", json!({
                "type": "Semantic Embedding",
                "model": "text-embedding-3",
                "dimensions": 1536,
            }))
            .add_edge_by_label("root", "ingest", BaseRelationshipType::Sequence)
            .add_edge_by_label("ingest", "extract", BaseRelationshipType::Sequence)
            .add_edge_by_label("extract", "analyze", BaseRelationshipType::Sequence)
            .add_edge_by_label("analyze", "embed", BaseRelationshipType::Sequence)
    }
}

// Graph domain compositions (when feature enabled)
#[cfg(feature = "graph")]
pub mod graph {
    use super::*;
    use cim_domain::AggregateRoot;
    use cim_domain_graph::aggregate::ConceptGraph;

    impl Composable for ConceptGraph {
        fn to_graph(&self) -> GraphComposition {
            let mut graph = GraphComposition::aggregate("ConceptGraph", self.id().to_string());

            // Store node ID mappings for edges
            let mut node_id_map = std::collections::HashMap::new();

            // Add nodes from the concept graph
            for (domain_node_id, node) in self.nodes() {
                let local_node_id = NodeId::new();
                let node_label = format!("node_{}", domain_node_id);

                graph = graph.add_node_with_id(
                    local_node_id,
                    BaseNodeType::Custom("Concept".to_string()),
                    &node_label,
                    json!({
                        "id": domain_node_id.to_string(),
                        "label": node.label,
                        "concept_type": format!("{:?}", node.concept_type),
                        "properties": node.properties,
                    })
                );

                // Store mapping from domain NodeId to local NodeId
                node_id_map.insert(domain_node_id, local_node_id);
            }

            // Add relationships using the mapped node IDs
            for (_edge_id, relationship) in self.relationships() {
                // Look up the local node IDs for source and target
                if let (Some(&source_id), Some(&target_id)) = (
                    node_id_map.get(&relationship.source_node_id),
                    node_id_map.get(&relationship.target_node_id)
                ) {
                    graph = graph.add_edge(
                        source_id,
                        target_id,
                        BaseRelationshipType::Custom(format!("{:?}", relationship.relationship_type)),
                    );
                }
            }

            graph
        }
    }
}

// Person domain compositions (when feature enabled)
#[cfg(feature = "person")]
pub mod person {
    use super::*;
    use cim_domain::AggregateRoot;
    use cim_domain_person::aggregate::{Person, IdentityComponent, ContactComponent};

    impl Composable for Person {
        fn to_graph(&self) -> GraphComposition {
            let mut graph = GraphComposition::aggregate("Person", self.id().to_string());

            // Add identity component if available
            if let Some(identity) = self.get_component::<IdentityComponent>() {
                graph = graph.add_node(
                    BaseNodeType::Value,
                    "identity",
                    json!({
                        "legal_name": identity.legal_name,
                        "preferred_name": identity.preferred_name,
                        "date_of_birth": identity.date_of_birth.map(|d| d.to_string()),
                        "government_id": identity.government_id.is_some(),
                    })
                );
                graph = graph.add_edge_by_label("root", "identity", BaseRelationshipType::Contains);
            }

            // Add contact component if available
            if let Some(contact) = self.get_component::<ContactComponent>() {
                graph = graph.add_node(
                    BaseNodeType::Value,
                    "contact",
                    json!({
                        "emails": contact.emails.iter().map(|e| e.email.clone()).collect::<Vec<_>>(),
                        "phones": contact.phones.iter().map(|p| p.number.clone()).collect::<Vec<_>>(),
                        "addresses": contact.addresses.len(),
                    })
                );
                graph = graph.add_edge_by_label("root", "contact", BaseRelationshipType::Contains);
            }

            graph
        }
    }
}

// Workflow domain compositions (when feature enabled)
#[cfg(feature = "workflow")]
pub mod workflow {
    use super::*;
    use cim_domain::AggregateRoot;
    use cim_domain_workflow::aggregate::WorkflowAggregate;
    use cim_domain_workflow::{WorkflowState, TransitionInput, TransitionOutput};

    impl<S, I, O> Composable for WorkflowAggregate<S, I, O>
    where
        S: WorkflowState,
        I: TransitionInput,
        O: TransitionOutput,
    {
        fn to_graph(&self) -> GraphComposition {
            let mut graph = GraphComposition::aggregate("Workflow", self.id().to_string());

            // Add current state
            graph = graph.add_node(
                BaseNodeType::Custom("State".to_string()),
                "current_state",
                json!({
                    "name": self.current_state().name(),
                    "is_terminal": self.current_state().is_terminal(),
                })
            );

            // Add workflow metadata
            graph = graph.add_node(
                BaseNodeType::Value,
                "metadata",
                json!({
                    "status": format!("{:?}", self.status()),
                    "started_at": self.started_at.elapsed().unwrap().as_secs(),
                    "transition_count": self.transition_count(),
                })
            );

            graph = graph.add_edge_by_label("root", "current_state", BaseRelationshipType::Contains);
            graph = graph.add_edge_by_label("root", "metadata", BaseRelationshipType::Contains);

            graph
        }
    }
}

// Location domain compositions (when feature enabled)
#[cfg(feature = "location")]
pub mod location {
    use super::*;
    use cim_domain::AggregateRoot;
    use cim_domain_location::aggregate::Location;

    impl Composable for Location {
        fn to_graph(&self) -> GraphComposition {
            let mut graph = GraphComposition::aggregate("Location", self.id().to_string());

            // Add basic location info
            graph = graph.add_node(
                BaseNodeType::Value,
                "info",
                json!({
                    "name": self.name,
                    "location_type": format!("{:?}", self.location_type),
                })
            );
            graph = graph.add_edge_by_label("root", "info", BaseRelationshipType::Contains);

            // Add address if physical
            if let Some(address) = &self.address {
                graph = graph.add_node(
                    BaseNodeType::Value,
                    "address",
                    json!({
                        "street": address.street1,
                        "city": address.locality,
                        "region": address.region,
                        "country": address.country,
                        "postal_code": address.postal_code,
                    })
                );
                graph = graph.add_edge_by_label("root", "address", BaseRelationshipType::Contains);
            }

            // Add coordinates if available
            if let Some(coords) = &self.coordinates {
                graph = graph.add_node(
                    BaseNodeType::Value,
                    "coordinates",
                    json!({
                        "latitude": coords.latitude,
                        "longitude": coords.longitude,
                        "altitude": coords.altitude,
                    })
                );
                graph = graph.add_edge_by_label("root", "coordinates", BaseRelationshipType::Contains);
            }

            graph
        }
    }
}

// Agent domain compositions (when feature enabled)
#[cfg(feature = "agent")]
pub mod agent {
    use super::*;
    use cim_domain::AggregateRoot;
    use cim_domain_agent::aggregate::{
        Agent, AgentMetadata, CapabilitiesComponent,
        PermissionsComponent, ToolAccessComponent,
    };

    impl Composable for Agent {
        fn to_graph(&self) -> GraphComposition {
            let mut graph = GraphComposition::aggregate("Agent", self.id().to_string());

            // Add basic agent info
            graph = graph.add_node(
                BaseNodeType::Value,
                "info",
                json!({
                    "agent_type": self.agent_type().to_string(),
                    "status": format!("{:?}", self.status()),
                    "owner_id": self.owner_id().to_string(),
                })
            );
            graph = graph.add_edge_by_label("root", "info", BaseRelationshipType::Contains);

            // Add metadata if available
            if let Some(metadata) = self.get_component::<AgentMetadata>() {
                graph = graph.add_node(
                    BaseNodeType::Value,
                    "metadata",
                    json!({
                        "name": metadata.name,
                        "description": metadata.description,
                        "tags": metadata.tags,
                        "created_at": metadata.created_at.to_rfc3339(),
                        "last_active": metadata.last_active.map(|t| t.to_rfc3339()),
                    })
                );
                graph = graph.add_edge_by_label("root", "metadata", BaseRelationshipType::Contains);
            }

            // Add capabilities if available
            if let Some(capabilities) = self.get_component::<CapabilitiesComponent>() {
                graph = graph.add_node(
                    BaseNodeType::Value,
                    "capabilities",
                    json!({
                        "capabilities": capabilities.capabilities,
                        "count": capabilities.capabilities.len(),
                    })
                );
                graph = graph.add_edge_by_label("root", "capabilities", BaseRelationshipType::Contains);
            }

            // Add permissions if available
            if let Some(permissions) = self.get_component::<PermissionsComponent>() {
                graph = graph.add_node(
                    BaseNodeType::Value,
                    "permissions",
                    json!({
                        "granted": permissions.permissions,
                        "denied": permissions.denials,
                        "roles": permissions.roles,
                    })
                );
                graph = graph.add_edge_by_label("root", "permissions", BaseRelationshipType::Contains);
            }

            // Add tool access if available
            if let Some(tools) = self.get_component::<ToolAccessComponent>() {
                graph = graph.add_node(
                    BaseNodeType::Value,
                    "tools",
                    json!({
                        "available_tools": tools.tools.keys().cloned().collect::<Vec<_>>(),
                        "tool_count": tools.tools.len(),
                    })
                );
                graph = graph.add_edge_by_label("root", "tools", BaseRelationshipType::Contains);
            }

            graph
        }
    }

    /// Create an agent capability graph
    pub fn create_agent_network() -> GraphComposition {
        GraphComposition::composite("AgentNetwork")
            .add_node(BaseNodeType::Custom("AgentType".to_string()), "human_agents", json!({
                "type": "Human Agents",
                "description": "Human-controlled agents in the system",
            }))
            .add_node(BaseNodeType::Custom("AgentType".to_string()), "ai_agents", json!({
                "type": "AI Agents",
                "description": "AI/ML model agents",
            }))
            .add_node(BaseNodeType::Custom("AgentType".to_string()), "system_agents", json!({
                "type": "System Agents",
                "description": "System/service agents",
            }))
            .add_node(BaseNodeType::Custom("Capability".to_string()), "data_processing", json!({
                "name": "Data Processing",
                "description": "Ability to process and transform data",
            }))
            .add_node(BaseNodeType::Custom("Capability".to_string()), "decision_making", json!({
                "name": "Decision Making",
                "description": "Ability to make autonomous decisions",
            }))
            .add_edge_by_label("root", "human_agents", BaseRelationshipType::Contains)
            .add_edge_by_label("root", "ai_agents", BaseRelationshipType::Contains)
            .add_edge_by_label("root", "system_agents", BaseRelationshipType::Contains)
            .add_edge_by_label("ai_agents", "data_processing", BaseRelationshipType::Custom("has_capability".to_string()))
            .add_edge_by_label("ai_agents", "decision_making", BaseRelationshipType::Custom("has_capability".to_string()))
    }
}

// Organization domain compositions (when feature enabled)
#[cfg(feature = "organization")]
pub mod organization {
    use super::*;
    use cim_domain::AggregateRoot;
    use cim_domain_organization::organization::{
        Organization, OrganizationMetadata, BudgetComponent,
    };

    impl Composable for Organization {
        fn to_graph(&self) -> GraphComposition {
            let mut graph = GraphComposition::aggregate("Organization", self.id().to_string());

            // Add basic organization info
            graph = graph.add_node(
                BaseNodeType::Value,
                "info",
                json!({
                    "name": self.name,
                    "type": format!("{:?}", self.org_type),
                    "status": format!("{:?}", self.status),
                    "member_count": self.member_count(),
                    "location_count": self.location_count(),
                })
            );
            graph = graph.add_edge_by_label("root", "info", BaseRelationshipType::Contains);

            // Add parent relationship if exists
            if let Some(parent_id) = self.parent_id {
                graph = graph.add_node(
                    BaseNodeType::Entity,
                    "parent",
                    json!({
                        "parent_id": parent_id.to_string(),
                    })
                );
                graph = graph.add_edge_by_label("root", "parent", BaseRelationshipType::Custom("reports_to".to_string()));
            }

            // Add child units
            for (idx, child_id) in self.child_units.iter().enumerate() {
                let child_label = format!("child_{}", idx);
                graph = graph.add_node(
                    BaseNodeType::Entity,
                    &child_label,
                    json!({
                        "child_id": child_id.to_string(),
                    })
                );
                graph = graph.add_edge_by_label("root", &child_label, BaseRelationshipType::Custom("manages".to_string()));
            }

            // Add metadata if available
            if let Some(metadata) = self.components.get::<OrganizationMetadata>() {
                graph = graph.add_node(
                    BaseNodeType::Value,
                    "metadata",
                    json!({
                        "industry": metadata.industry,
                        "size_category": metadata.size_category.as_ref().map(|s| format!("{:?}", s)),
                        "founded_date": metadata.founded_date.map(|d| d.to_string()),
                        "website": metadata.website,
                    })
                );
                graph = graph.add_edge_by_label("root", "metadata", BaseRelationshipType::Contains);
            }

            // Add budget if available
            if let Some(budget) = self.components.get::<BudgetComponent>() {
                graph = graph.add_node(
                    BaseNodeType::Value,
                    "budget",
                    json!({
                        "fiscal_year": budget.fiscal_year,
                        "total_budget": budget.total_budget,
                        "currency": budget.currency,
                        "allocated": budget.allocated,
                        "spent": budget.spent,
                        "remaining": budget.total_budget - budget.spent,
                    })
                );
                graph = graph.add_edge_by_label("root", "budget", BaseRelationshipType::Contains);
            }

            // Add primary location if set
            if let Some(location_id) = self.primary_location {
                graph = graph.add_node(
                    BaseNodeType::Entity,
                    "primary_location",
                    json!({
                        "location_id": location_id.to_string(),
                    })
                );
                graph = graph.add_edge_by_label("root", "primary_location", BaseRelationshipType::Custom("headquartered_at".to_string()));
            }

            graph
        }
    }

    /// Create an organizational hierarchy graph
    pub fn create_org_hierarchy() -> GraphComposition {
        GraphComposition::composite("OrganizationalHierarchy")
            .add_node(BaseNodeType::Custom("Level".to_string()), "company", json!({
                "level": "Company",
                "description": "Top-level organization",
            }))
            .add_node(BaseNodeType::Custom("Level".to_string()), "division", json!({
                "level": "Division",
                "description": "Major business divisions",
            }))
            .add_node(BaseNodeType::Custom("Level".to_string()), "department", json!({
                "level": "Department",
                "description": "Functional departments",
            }))
            .add_node(BaseNodeType::Custom("Level".to_string()), "team", json!({
                "level": "Team",
                "description": "Working teams",
            }))
            .add_edge_by_label("root", "company", BaseRelationshipType::Hierarchy)
            .add_edge_by_label("company", "division", BaseRelationshipType::Hierarchy)
            .add_edge_by_label("division", "department", BaseRelationshipType::Hierarchy)
            .add_edge_by_label("department", "team", BaseRelationshipType::Hierarchy)
    }
}

// Conceptual Spaces domain compositions (when feature enabled)
#[cfg(feature = "conceptualspaces")]
pub mod conceptualspaces {
    use super::*;
    use cim_domain::AggregateRoot;
    use cim_domain_conceptualspaces::{ConceptualSpaceAggregate, ConceptualPoint};

    impl Composable for ConceptualSpaceAggregate {
        fn to_graph(&self) -> GraphComposition {
            let mut graph = GraphComposition::aggregate("ConceptualSpace", self.id().to_string());

            // Add space metadata
            graph = graph.add_node(
                BaseNodeType::Value,
                "metadata",
                json!({
                    "name": "ConceptualSpace",
                    "dimensions": self.space().dimension_ids.len(),
                    "regions": self.space().regions.len(),
                    "points": self.space().points.len(),
                })
            );
            graph = graph.add_edge_by_label("root", "metadata", BaseRelationshipType::Contains);

            // Add dimensions
            for (idx, dim_id) in self.space().dimension_ids.iter().enumerate() {
                let dim_label = format!("dimension_{}", idx);
                graph = graph.add_node(
                    BaseNodeType::Value,
                    &dim_label,
                    json!({
                        "id": dim_id.0.to_string(),
                        "index": idx,
                    })
                );
                graph = graph.add_edge_by_label("root", &dim_label, BaseRelationshipType::Contains);
            }

            // Add regions
            for (region_id, region) in &self.space().regions {
                let region_label = format!("region_{}", region_id);
                graph = graph.add_node(
                    BaseNodeType::Custom("ConvexRegion".to_string()),
                    &region_label,
                    json!({
                        "id": region_id.to_string(),
                        "name": region.name,
                        "member_count": region.member_points.len(),
                    })
                );
                graph = graph.add_edge_by_label("root", &region_label, BaseRelationshipType::Contains);
            }

            graph
        }
    }

    /// Create a conceptual space visualization
    pub fn create_conceptual_space_viz() -> GraphComposition {
        GraphComposition::composite("ConceptualSpaceVisualization")
            .add_node(BaseNodeType::Custom("Space".to_string()), "space", json!({
                "type": "Conceptual Space",
                "description": "High-dimensional semantic space",
            }))
            .add_node(BaseNodeType::Custom("Dimension".to_string()), "quality_dims", json!({
                "type": "Quality Dimensions",
                "description": "Dimensions representing quality aspects",
            }))
            .add_node(BaseNodeType::Custom("Region".to_string()), "concepts", json!({
                "type": "Concept Regions",
                "description": "Convex regions representing natural categories",
            }))
            .add_edge_by_label("root", "space", BaseRelationshipType::Contains)
            .add_edge_by_label("space", "quality_dims", BaseRelationshipType::Contains)
            .add_edge_by_label("space", "concepts", BaseRelationshipType::Contains)
    }
}

/// Compose multiple domain objects into a knowledge graph
pub fn compose_knowledge_graph<T: Composable>(objects: &[T]) -> GraphComposition {
    if objects.is_empty() {
        return GraphComposition::composite("KnowledgeGraph");
    }

    let mut graph = objects[0].to_graph();

    for obj in &objects[1..] {
        graph = graph.parallel(&obj.to_graph())
            .unwrap_or_else(|_| graph.clone());
    }

    graph
}
