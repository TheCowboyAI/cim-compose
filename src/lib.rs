//! Graph Composition Library
//!
//! A library for composable graph structures based on category theory.
//! This library provides the foundation for composing domain models (from cim-domain)
//! into graphs that can be transformed and reasoned about mathematically.
//!
//! ## Architecture
//!
//! - **cim-domain**: Provides core DDD building blocks (Entity, Aggregate, etc.)
//! - **cim-compose**: Provides graph composition of those building blocks (this crate)
//! - **Domain modules**: Pure domain logic (Document, Graph, Person, etc.)
//!
//! ## Key Concepts
//!
//! - **GraphComposition**: The main type for representing composable graphs
//! - **CompositionNode**: Nodes within a graph (can reference domain entities)
//! - **CompositionEdge**: Relationships between nodes
//! - **Category Theory Operations**: Morphisms, Functors, and Monads for graph transformation
//! - **Domain Compositions**: Feature-gated traits for composing specific domain aggregates

pub mod base_types;
pub mod composition;
pub mod mapping;
pub mod domain_compositions;

// Re-export main types
pub use base_types::*;
pub use composition::*;
pub use mapping::*;
pub use domain_compositions::{Composable, Decomposable};
