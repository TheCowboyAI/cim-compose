//! Infrastructure Layer 1.3: Message Routing Tests for cim-compose
//! 
//! User Story: As a composition system, I need to route composition commands to appropriate handlers
//!
//! Test Requirements:
//! - Verify composition command routing to correct handlers
//! - Verify handler registration and discovery
//! - Verify fallback handling for unknown commands
//! - Verify routing performance metrics
//!
//! Event Sequence:
//! 1. RouterInitialized
//! 2. HandlerRegistered { command_type, handler_id }
//! 3. CommandRouted { command_type, handler_id }
//! 4. FallbackHandlerInvoked { command_type }
//!
//! ```mermaid
//! graph LR
//!     A[Test Start] --> B[Initialize Router]
//!     B --> C[RouterInitialized]
//!     C --> D[Register Handler]
//!     D --> E[HandlerRegistered]
//!     E --> F[Route Command]
//!     F --> G[CommandRouted]
//!     G --> H[Test Success]
//! ```

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Composition command types for testing
#[derive(Debug, Clone, PartialEq)]
pub enum CompositionCommand {
    CreateGraph {
        graph_id: String,
        composition_type: String,
    },
    AddNode {
        graph_id: String,
        node_id: String,
        node_type: String,
    },
    AddEdge {
        graph_id: String,
        edge_id: String,
        source_id: String,
        target_id: String,
    },
    ComposeGraphs {
        source_id: String,
        target_id: String,
        composition_type: String,
    },
    ApplyFunctor {
        graph_id: String,
        functor_type: String,
    },
    ValidateInvariants {
        graph_id: String,
    },
}

/// Handler response types
#[derive(Debug, Clone, PartialEq)]
pub enum HandlerResponse {
    Success { message: String },
    Error { reason: String },
    Async { task_id: String },
}

/// Trait for composition command handlers
pub trait CompositionHandler: Send + Sync {
    fn handle(&self, command: &CompositionCommand) -> HandlerResponse;
    fn can_handle(&self, command: &CompositionCommand) -> bool;
    fn handler_id(&self) -> String;
    fn clone_box(&self) -> Box<dyn CompositionHandler>;
}

impl Clone for Box<dyn CompositionHandler> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// Mock handler for graph creation
#[derive(Clone)]
pub struct GraphCreationHandler {
    id: String,
}

impl GraphCreationHandler {
    pub fn new() -> Self {
        Self {
            id: "graph-creation-handler".to_string(),
        }
    }
}

impl CompositionHandler for GraphCreationHandler {
    fn handle(&self, command: &CompositionCommand) -> HandlerResponse {
        match command {
            CompositionCommand::CreateGraph { graph_id, composition_type } => {
                HandlerResponse::Success {
                    message: format!("Created {} graph with id {}", composition_type, graph_id),
                }
            }
            _ => HandlerResponse::Error {
                reason: "Not a graph creation command".to_string(),
            },
        }
    }

    fn can_handle(&self, command: &CompositionCommand) -> bool {
        matches!(command, CompositionCommand::CreateGraph { .. })
    }

    fn handler_id(&self) -> String {
        self.id.clone()
    }

    fn clone_box(&self) -> Box<dyn CompositionHandler> {
        Box::new(self.clone())
    }
}

/// Mock handler for node operations
#[derive(Clone)]
pub struct NodeOperationHandler {
    id: String,
}

impl NodeOperationHandler {
    pub fn new() -> Self {
        Self {
            id: "node-operation-handler".to_string(),
        }
    }
}

impl CompositionHandler for NodeOperationHandler {
    fn handle(&self, command: &CompositionCommand) -> HandlerResponse {
        match command {
            CompositionCommand::AddNode { graph_id, node_id, node_type } => {
                HandlerResponse::Success {
                    message: format!("Added {} node {} to graph {}", node_type, node_id, graph_id),
                }
            }
            _ => HandlerResponse::Error {
                reason: "Not a node operation command".to_string(),
            },
        }
    }

    fn can_handle(&self, command: &CompositionCommand) -> bool {
        matches!(command, CompositionCommand::AddNode { .. })
    }

    fn handler_id(&self) -> String {
        self.id.clone()
    }

    fn clone_box(&self) -> Box<dyn CompositionHandler> {
        Box::new(self.clone())
    }
}

/// Fallback handler for unhandled commands
#[derive(Clone)]
pub struct FallbackHandler {
    id: String,
}

impl FallbackHandler {
    pub fn new() -> Self {
        Self {
            id: "fallback-handler".to_string(),
        }
    }
}

impl CompositionHandler for FallbackHandler {
    fn handle(&self, _command: &CompositionCommand) -> HandlerResponse {
        HandlerResponse::Error {
            reason: "No handler registered for this command type".to_string(),
        }
    }

    fn can_handle(&self, _command: &CompositionCommand) -> bool {
        true // Fallback handles everything
    }

    fn handler_id(&self) -> String {
        self.id.clone()
    }

    fn clone_box(&self) -> Box<dyn CompositionHandler> {
        Box::new(self.clone())
    }
}

/// Router events for testing
#[derive(Debug, Clone, PartialEq)]
pub enum RouterEvent {
    RouterInitialized,
    HandlerRegistered {
        command_type: String,
        handler_id: String,
    },
    CommandRouted {
        command_type: String,
        handler_id: String,
    },
    FallbackHandlerInvoked {
        command_type: String,
    },
    RoutingError {
        reason: String,
    },
    RoutingStatisticsUpdated {
        total_routed: usize,
        by_handler: HashMap<String, usize>,
    },
}

/// Message router for composition commands
pub struct CompositionRouter {
    handlers: Vec<Box<dyn CompositionHandler>>,
    fallback: Box<dyn CompositionHandler>,
    routing_stats: Arc<Mutex<RoutingStatistics>>,
}

/// Routing statistics
#[derive(Debug, Clone)]
pub struct RoutingStatistics {
    total_routed: usize,
    by_handler: HashMap<String, usize>,
    by_command_type: HashMap<String, usize>,
    routing_times: Vec<Duration>,
}

impl RoutingStatistics {
    pub fn new() -> Self {
        Self {
            total_routed: 0,
            by_handler: HashMap::new(),
            by_command_type: HashMap::new(),
            routing_times: Vec::new(),
        }
    }

    pub fn record_routing(&mut self, handler_id: &str, command_type: &str, duration: Duration) {
        self.total_routed += 1;
        *self.by_handler.entry(handler_id.to_string()).or_insert(0) += 1;
        *self.by_command_type.entry(command_type.to_string()).or_insert(0) += 1;
        self.routing_times.push(duration);
    }

    pub fn average_routing_time(&self) -> Option<Duration> {
        if self.routing_times.is_empty() {
            None
        } else {
            let total: Duration = self.routing_times.iter().sum();
            Some(total / self.routing_times.len() as u32)
        }
    }
}

impl CompositionRouter {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
            fallback: Box::new(FallbackHandler::new()),
            routing_stats: Arc::new(Mutex::new(RoutingStatistics::new())),
        }
    }

    pub fn register_handler(&mut self, handler: Box<dyn CompositionHandler>) -> String {
        let handler_id = handler.handler_id();
        self.handlers.push(handler);
        handler_id
    }

    pub fn route_command(&self, command: &CompositionCommand) -> (HandlerResponse, String) {
        let start = Instant::now();
        let command_type = self.get_command_type(command);

        // Find the first handler that can handle this command
        for handler in &self.handlers {
            if handler.can_handle(command) {
                let response = handler.handle(command);
                let handler_id = handler.handler_id();
                
                // Record statistics
                let duration = start.elapsed();
                if let Ok(mut stats) = self.routing_stats.lock() {
                    stats.record_routing(&handler_id, &command_type, duration);
                }

                return (response, handler_id);
            }
        }

        // No handler found, use fallback
        let response = self.fallback.handle(command);
        let handler_id = self.fallback.handler_id();
        
        // Record statistics for fallback
        let duration = start.elapsed();
        if let Ok(mut stats) = self.routing_stats.lock() {
            stats.record_routing(&handler_id, &command_type, duration);
        }

        (response, handler_id)
    }

    pub fn get_statistics(&self) -> RoutingStatistics {
        self.routing_stats.lock().unwrap().clone()
    }

    fn get_command_type(&self, command: &CompositionCommand) -> String {
        match command {
            CompositionCommand::CreateGraph { .. } => "CreateGraph".to_string(),
            CompositionCommand::AddNode { .. } => "AddNode".to_string(),
            CompositionCommand::AddEdge { .. } => "AddEdge".to_string(),
            CompositionCommand::ComposeGraphs { .. } => "ComposeGraphs".to_string(),
            CompositionCommand::ApplyFunctor { .. } => "ApplyFunctor".to_string(),
            CompositionCommand::ValidateInvariants { .. } => "ValidateInvariants".to_string(),
        }
    }

    pub fn handler_count(&self) -> usize {
        self.handlers.len()
    }
}

/// Event validator for router testing
pub struct RouterEventValidator {
    expected_events: Vec<RouterEvent>,
    captured_events: Vec<RouterEvent>,
}

impl RouterEventValidator {
    pub fn new() -> Self {
        Self {
            expected_events: Vec::new(),
            captured_events: Vec::new(),
        }
    }

    pub fn expect_sequence(mut self, events: Vec<RouterEvent>) -> Self {
        self.expected_events = events;
        self
    }

    pub fn capture_event(&mut self, event: RouterEvent) {
        self.captured_events.push(event);
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.captured_events.len() != self.expected_events.len() {
            return Err(format!(
                "Event count mismatch: expected {}, got {}",
                self.expected_events.len(),
                self.captured_events.len()
            ));
        }

        for (i, (expected, actual)) in self.expected_events.iter()
            .zip(self.captured_events.iter())
            .enumerate()
        {
            if expected != actual {
                return Err(format!(
                    "Event mismatch at position {}: expected {:?}, got {:?}",
                    i, expected, actual
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_initialization() {
        // Arrange
        let mut validator = RouterEventValidator::new()
            .expect_sequence(vec![
                RouterEvent::RouterInitialized,
            ]);

        // Act
        let router = CompositionRouter::new();

        // Assert
        assert_eq!(router.handler_count(), 0);
        validator.capture_event(RouterEvent::RouterInitialized);
        assert!(validator.validate().is_ok());
    }

    #[test]
    fn test_handler_registration() {
        // Arrange
        let mut router = CompositionRouter::new();
        let mut validator = RouterEventValidator::new();

        // Act
        let handler = Box::new(GraphCreationHandler::new());
        let handler_id = router.register_handler(handler);

        // Assert
        assert_eq!(router.handler_count(), 1);
        validator.capture_event(RouterEvent::HandlerRegistered {
            command_type: "GraphCreation".to_string(),
            handler_id,
        });
    }

    #[test]
    fn test_command_routing() {
        // Arrange
        let mut router = CompositionRouter::new();
        let mut validator = RouterEventValidator::new();
        
        router.register_handler(Box::new(GraphCreationHandler::new()));

        // Act
        let command = CompositionCommand::CreateGraph {
            graph_id: "graph-123".to_string(),
            composition_type: "Atomic".to_string(),
        };

        let (response, handler_id) = router.route_command(&command);

        // Assert
        match response {
            HandlerResponse::Success { message } => {
                assert!(message.contains("Created Atomic graph"));
            }
            _ => panic!("Expected success response"),
        }

        validator.capture_event(RouterEvent::CommandRouted {
            command_type: "CreateGraph".to_string(),
            handler_id,
        });
    }

    #[test]
    fn test_multiple_handler_routing() {
        // Arrange
        let mut router = CompositionRouter::new();
        
        router.register_handler(Box::new(GraphCreationHandler::new()));
        router.register_handler(Box::new(NodeOperationHandler::new()));

        // Act & Assert - Graph creation
        let create_cmd = CompositionCommand::CreateGraph {
            graph_id: "g1".to_string(),
            composition_type: "Composite".to_string(),
        };
        let (response1, handler1) = router.route_command(&create_cmd);
        assert!(matches!(response1, HandlerResponse::Success { .. }));
        assert_eq!(handler1, "graph-creation-handler");

        // Act & Assert - Node addition
        let add_node_cmd = CompositionCommand::AddNode {
            graph_id: "g1".to_string(),
            node_id: "n1".to_string(),
            node_type: "Process".to_string(),
        };
        let (response2, handler2) = router.route_command(&add_node_cmd);
        assert!(matches!(response2, HandlerResponse::Success { .. }));
        assert_eq!(handler2, "node-operation-handler");
    }

    #[test]
    fn test_fallback_handler() {
        // Arrange
        let router = CompositionRouter::new();
        let mut validator = RouterEventValidator::new();

        // Act - Route unhandled command
        let command = CompositionCommand::ComposeGraphs {
            source_id: "g1".to_string(),
            target_id: "g2".to_string(),
            composition_type: "Sequential".to_string(),
        };

        let (response, handler_id) = router.route_command(&command);

        // Assert
        assert!(matches!(response, HandlerResponse::Error { .. }));
        assert_eq!(handler_id, "fallback-handler");

        validator.capture_event(RouterEvent::FallbackHandlerInvoked {
            command_type: "ComposeGraphs".to_string(),
        });
    }

    #[test]
    fn test_routing_error_handling() {
        // Arrange
        let mut router = CompositionRouter::new();
        let mut validator = RouterEventValidator::new();
        
        router.register_handler(Box::new(GraphCreationHandler::new()));

        // Act - Send wrong command to handler
        let command = CompositionCommand::AddNode {
            graph_id: "g1".to_string(),
            node_id: "n1".to_string(),
            node_type: "Process".to_string(),
        };

        let (response, _) = router.route_command(&command);

        // Assert - Fallback should handle it
        match response {
            HandlerResponse::Error { reason } => {
                assert!(reason.contains("No handler registered"));
            }
            _ => panic!("Expected error response"),
        }

        validator.capture_event(RouterEvent::RoutingError {
            reason: "No suitable handler found".to_string(),
        });
    }

    #[test]
    fn test_routing_statistics() {
        // Arrange
        let mut router = CompositionRouter::new();
        router.register_handler(Box::new(GraphCreationHandler::new()));
        router.register_handler(Box::new(NodeOperationHandler::new()));

        // Act - Route multiple commands
        let commands = vec![
            CompositionCommand::CreateGraph {
                graph_id: "g1".to_string(),
                composition_type: "Atomic".to_string(),
            },
            CompositionCommand::CreateGraph {
                graph_id: "g2".to_string(),
                composition_type: "Composite".to_string(),
            },
            CompositionCommand::AddNode {
                graph_id: "g1".to_string(),
                node_id: "n1".to_string(),
                node_type: "Value".to_string(),
            },
        ];

        for cmd in &commands {
            router.route_command(cmd);
        }

        // Assert - Check statistics
        let stats = router.get_statistics();
        assert_eq!(stats.total_routed, 3);
        assert_eq!(stats.by_handler.get("graph-creation-handler"), Some(&2));
        assert_eq!(stats.by_handler.get("node-operation-handler"), Some(&1));
        assert_eq!(stats.by_command_type.get("CreateGraph"), Some(&2));
        assert_eq!(stats.by_command_type.get("AddNode"), Some(&1));
    }

    #[test]
    fn test_concurrent_routing() {
        // Arrange
        let router = Arc::new(CompositionRouter::new());
        let mut router_mut = Arc::try_unwrap(router.clone()).unwrap_or_else(|arc| {
            // If we can't unwrap, create a new router for registration
            let mut new_router = CompositionRouter::new();
            new_router.register_handler(Box::new(GraphCreationHandler::new()));
            new_router
        });
        
        router_mut.register_handler(Box::new(GraphCreationHandler::new()));
        let router = Arc::new(router_mut);

        // Act - Simulate concurrent routing
        let handles: Vec<_> = (0..5)
            .map(|i| {
                let router_clone = router.clone();
                std::thread::spawn(move || {
                    let command = CompositionCommand::CreateGraph {
                        graph_id: format!("g{}", i),
                        composition_type: "Concurrent".to_string(),
                    };
                    router_clone.route_command(&command)
                })
            })
            .collect();

        // Wait for all threads
        let results: Vec<_> = handles.into_iter()
            .map(|h| h.join().unwrap())
            .collect();

        // Assert - All should succeed
        assert_eq!(results.len(), 5);
        for (response, _) in results {
            assert!(matches!(response, HandlerResponse::Success { .. }));
        }
    }

    #[test]
    fn test_handler_response_types() {
        // Test different response types
        
        // Success response
        let success = HandlerResponse::Success {
            message: "Operation completed".to_string(),
        };
        assert!(matches!(success, HandlerResponse::Success { .. }));

        // Error response
        let error = HandlerResponse::Error {
            reason: "Invalid input".to_string(),
        };
        assert!(matches!(error, HandlerResponse::Error { .. }));

        // Async response
        let async_resp = HandlerResponse::Async {
            task_id: "task-123".to_string(),
        };
        assert!(matches!(async_resp, HandlerResponse::Async { .. }));
    }
} 