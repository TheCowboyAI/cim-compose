//! Infrastructure Layer 1.2: Event Stream Tests for cim-compose
//! 
//! User Story: As a composition system, I need to persist composition events with CID chains for integrity
//!
//! Test Requirements:
//! - Verify composition event persistence with CID calculation
//! - Verify CID chain integrity for composition transitions
//! - Verify composition event replay from store
//! - Verify composition snapshot creation and restoration
//!
//! Event Sequence:
//! 1. CompositionEventStoreInitialized
//! 2. CompositionEventPersisted { event_id, cid, previous_cid }
//! 3. CIDChainValidated { start_cid, end_cid, length }
//! 4. CompositionEventsReplayed { count, graph_id }
//!
//! ```mermaid
//! graph LR
//!     A[Test Start] --> B[Initialize Store]
//!     B --> C[CompositionEventStoreInitialized]
//!     C --> D[Create Composition Event]
//!     D --> E[CompositionEventPersisted]
//!     E --> F[Validate CID Chain]
//!     F --> G[CIDChainValidated]
//!     G --> H[Replay Events]
//!     H --> I[CompositionEventsReplayed]
//!     I --> J[Test Success]
//! ```

use std::collections::HashMap;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};

/// Mock CID representation for testing
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Cid(String);

impl Cid {
    pub fn new(data: &[u8]) -> Self {
        // Simple mock CID calculation
        let hash = data.iter().fold(0u64, |acc, &b| acc.wrapping_add(b as u64));
        Self(format!("Qm{:x}", hash))
    }
}

/// Composition domain events for testing
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CompositionDomainEvent {
    GraphCreated {
        graph_id: String,
        composition_type: String,
        root_node_id: String,
        timestamp: SystemTime,
    },
    NodeAdded {
        graph_id: String,
        node_id: String,
        node_type: String,
        label: String,
        timestamp: SystemTime,
    },
    EdgeAdded {
        graph_id: String,
        edge_id: String,
        source_id: String,
        target_id: String,
        relationship: String,
        timestamp: SystemTime,
    },
    GraphComposed {
        graph_id: String,
        source_graph_id: String,
        target_graph_id: String,
        composition_type: String,
        timestamp: SystemTime,
    },
    InvariantAdded {
        graph_id: String,
        invariant_id: String,
        description: String,
        timestamp: SystemTime,
    },
}

/// Event store events for testing
#[derive(Debug, Clone, PartialEq)]
pub enum CompositionEventStoreEvent {
    CompositionEventStoreInitialized,
    CompositionEventPersisted {
        event_id: String,
        cid: Cid,
        previous_cid: Option<Cid>,
    },
    CIDChainValidated {
        start_cid: Cid,
        end_cid: Cid,
        length: usize,
    },
    CompositionEventsReplayed {
        count: usize,
        graph_id: String,
    },
    SnapshotCreated {
        snapshot_cid: Cid,
        event_count: usize,
    },
    SnapshotRestored {
        snapshot_cid: Cid,
        restored_count: usize,
    },
}

/// Event with CID chain
#[derive(Debug, Clone)]
pub struct ChainedCompositionEvent {
    pub event_id: String,
    pub event: CompositionDomainEvent,
    pub cid: Cid,
    pub previous_cid: Option<Cid>,
    pub sequence: u64,
}

/// Mock event store for composition events
pub struct MockCompositionEventStore {
    events: Vec<ChainedCompositionEvent>,
    snapshots: HashMap<Cid, Vec<ChainedCompositionEvent>>,
}

impl MockCompositionEventStore {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            snapshots: HashMap::new(),
        }
    }

    pub fn append_event(
        &mut self,
        event: CompositionDomainEvent,
    ) -> Result<(String, Cid, Option<Cid>), String> {
        let event_id = format!("evt_{}", self.events.len());
        let previous_cid = self.events.last().map(|e| e.cid.clone());
        
        // Calculate CID including previous CID
        let event_data = format!("{:?}{:?}", event, previous_cid);
        let cid = Cid::new(event_data.as_bytes());
        
        let sequence = self.events.len() as u64;
        
        let chained_event = ChainedCompositionEvent {
            event_id: event_id.clone(),
            event,
            cid: cid.clone(),
            previous_cid: previous_cid.clone(),
            sequence,
        };
        
        self.events.push(chained_event);
        
        Ok((event_id, cid, previous_cid))
    }

    pub fn validate_chain(&self) -> Result<(Cid, Cid, usize), String> {
        if self.events.is_empty() {
            return Err("No events to validate".to_string());
        }

        // Validate each event's CID chain
        for i in 1..self.events.len() {
            let current = &self.events[i];
            let previous = &self.events[i - 1];
            
            if current.previous_cid.as_ref() != Some(&previous.cid) {
                return Err(format!("Chain broken at sequence {i}: expected {:?}, got {:?}", previous.cid, current.previous_cid));
            }
        }

        let start_cid = self.events.first().unwrap().cid.clone();
        let end_cid = self.events.last().unwrap().cid.clone();
        let length = self.events.len();

        Ok((start_cid, end_cid, length))
    }

    pub fn replay_events(&self, graph_id: &str) -> Vec<ChainedCompositionEvent> {
        self.events
            .iter()
            .filter(|e| match &e.event {
                CompositionDomainEvent::GraphCreated { graph_id: id, .. } => id == graph_id,
                CompositionDomainEvent::NodeAdded { graph_id: id, .. } => id == graph_id,
                CompositionDomainEvent::EdgeAdded { graph_id: id, .. } => id == graph_id,
                CompositionDomainEvent::GraphComposed { graph_id: id, .. } => id == graph_id,
                CompositionDomainEvent::InvariantAdded { graph_id: id, .. } => id == graph_id,
            })
            .cloned()
            .collect()
    }

    pub fn create_snapshot(&mut self) -> Result<Cid, String> {
        if self.events.is_empty() {
            return Err("No events to snapshot".to_string());
        }

        let snapshot_data = format!("{:?}", self.events);
        let snapshot_cid = Cid::new(snapshot_data.as_bytes());
        
        self.snapshots.insert(snapshot_cid.clone(), self.events.clone());
        
        Ok(snapshot_cid)
    }

    pub fn restore_from_snapshot(&mut self, snapshot_cid: &Cid) -> Result<usize, String> {
        match self.snapshots.get(snapshot_cid) {
            Some(events) => {
                self.events = events.clone();
                Ok(events.len())
            }
            None => Err("Snapshot not found".to_string()),
        }
    }
}

/// Event stream validator for composition event store testing
pub struct CompositionEventStreamValidator {
    expected_events: Vec<CompositionEventStoreEvent>,
    captured_events: Vec<CompositionEventStoreEvent>,
}

impl CompositionEventStreamValidator {
    pub fn new() -> Self {
        Self {
            expected_events: Vec::new(),
            captured_events: Vec::new(),
        }
    }

    pub fn expect_sequence(mut self, events: Vec<CompositionEventStoreEvent>) -> Self {
        self.expected_events = events;
        self
    }

    pub fn capture_event(&mut self, event: CompositionEventStoreEvent) {
        self.captured_events.push(event);
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.captured_events.len() != self.expected_events.len() {
            return Err(format!("Event count mismatch: expected {}, got {}",
                self.expected_events.len(),
                self.captured_events.len()
            ));
        }

        for (i, (expected, actual)) in self.expected_events.iter()
            .zip(self.captured_events.iter())
            .enumerate()
        {
            if expected != actual {
                return Err(format!("Event mismatch at position {i}: expected {:?}, got {:?}", expected, actual));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_composition_event_store_initialization() {
        // Arrange
        let mut validator = CompositionEventStreamValidator::new()
            .expect_sequence(vec![
                CompositionEventStoreEvent::CompositionEventStoreInitialized,
            ]);

        // Act
        let store = MockCompositionEventStore::new();
        validator.capture_event(CompositionEventStoreEvent::CompositionEventStoreInitialized);

        // Assert
        assert!(validator.validate().is_ok());
        assert_eq!(store.events.len(), 0);
    }

    #[test]
    fn test_composition_event_persistence_with_cid() {
        // Arrange
        let mut store = MockCompositionEventStore::new();
        let mut validator = CompositionEventStreamValidator::new();

        // Act
        let event = CompositionDomainEvent::GraphCreated {
            graph_id: "graph-123".to_string(),
            composition_type: "Atomic".to_string(),
            root_node_id: "node-root".to_string(),
            timestamp: SystemTime::now(),
        };

        let (event_id, cid, previous_cid) = store.append_event(event).unwrap();

        // Assert
        assert!(previous_cid.is_none()); // First event has no previous
        assert!(!event_id.is_empty());
        
        validator.capture_event(CompositionEventStoreEvent::CompositionEventPersisted {
            event_id,
            cid,
            previous_cid,
        });
    }

    #[test]
    fn test_composition_lifecycle_cid_chain() {
        // Arrange
        let mut store = MockCompositionEventStore::new();
        let mut validator = CompositionEventStreamValidator::new();
        let graph_id = "graph-lifecycle";

        // Act - Create composition lifecycle events
        let event1 = CompositionDomainEvent::GraphCreated {
            graph_id: graph_id.to_string(),
            composition_type: "Composite".to_string(),
            root_node_id: "root-1".to_string(),
            timestamp: SystemTime::now(),
        };

        let event2 = CompositionDomainEvent::NodeAdded {
            graph_id: graph_id.to_string(),
            node_id: "node-1".to_string(),
            node_type: "Process".to_string(),
            label: "Process Node".to_string(),
            timestamp: SystemTime::now(),
        };

        let event3 = CompositionDomainEvent::NodeAdded {
            graph_id: graph_id.to_string(),
            node_id: "node-2".to_string(),
            node_type: "Decision".to_string(),
            label: "Decision Node".to_string(),
            timestamp: SystemTime::now(),
        };

        let event4 = CompositionDomainEvent::EdgeAdded {
            graph_id: graph_id.to_string(),
            edge_id: "edge-1".to_string(),
            source_id: "node-1".to_string(),
            target_id: "node-2".to_string(),
            relationship: "Sequence".to_string(),
            timestamp: SystemTime::now(),
        };

        let event5 = CompositionDomainEvent::InvariantAdded {
            graph_id: graph_id.to_string(),
            invariant_id: "inv-1".to_string(),
            description: "No cycles allowed".to_string(),
            timestamp: SystemTime::now(),
        };

        store.append_event(event1).unwrap();
        store.append_event(event2).unwrap();
        store.append_event(event3).unwrap();
        store.append_event(event4).unwrap();
        store.append_event(event5).unwrap();

        // Validate chain
        let (start_cid, end_cid, length) = store.validate_chain().unwrap();

        // Assert
        assert_eq!(length, 5);
        assert_ne!(start_cid, end_cid);
        
        validator.capture_event(CompositionEventStoreEvent::CIDChainValidated {
            start_cid,
            end_cid,
            length,
        });
    }

    #[test]
    fn test_composition_event_replay() {
        // Arrange
        let mut store = MockCompositionEventStore::new();
        let mut validator = CompositionEventStreamValidator::new();
        let graph_id = "graph-replay";

        // Add events for multiple graphs
        store.append_event(CompositionDomainEvent::GraphCreated {
            graph_id: graph_id.to_string(),
            composition_type: "Entity".to_string(),
            root_node_id: "entity-root".to_string(),
            timestamp: SystemTime::now(),
        }).unwrap();

        store.append_event(CompositionDomainEvent::GraphCreated {
            graph_id: "other-graph".to_string(),
            composition_type: "ValueObject".to_string(),
            root_node_id: "vo-root".to_string(),
            timestamp: SystemTime::now(),
        }).unwrap();

        store.append_event(CompositionDomainEvent::NodeAdded {
            graph_id: graph_id.to_string(),
            node_id: "entity-field".to_string(),
            node_type: "Property".to_string(),
            label: "Name".to_string(),
            timestamp: SystemTime::now(),
        }).unwrap();

        // Act
        let replayed = store.replay_events(graph_id);

        // Assert
        assert_eq!(replayed.len(), 2); // Only events for the specific graph
        
        validator.capture_event(CompositionEventStoreEvent::CompositionEventsReplayed {
            count: replayed.len(),
            graph_id: graph_id.to_string(),
        });
    }

    #[test]
    fn test_composition_snapshot_creation_and_restoration() {
        // Arrange
        let mut store = MockCompositionEventStore::new();
        let mut validator = CompositionEventStreamValidator::new();

        // Add some events
        for i in 0..3 {
            store.append_event(CompositionDomainEvent::GraphCreated {
                graph_id: format!("graph-{i}"),
                composition_type: "Aggregate".to_string(),
                root_node_id: format!("agg-{i}"),
                timestamp: SystemTime::now(),
            }).unwrap();
        }

        // Act - Create snapshot
        let snapshot_cid = store.create_snapshot().unwrap();
        
        validator.capture_event(CompositionEventStoreEvent::SnapshotCreated {
            snapshot_cid: snapshot_cid.clone(),
            event_count: 3,
        });

        // Clear events and restore
        store.events.clear();
        let restored_count = store.restore_from_snapshot(&snapshot_cid).unwrap();

        // Assert
        assert_eq!(restored_count, 3);
        assert_eq!(store.events.len(), 3);
        
        validator.capture_event(CompositionEventStoreEvent::SnapshotRestored {
            snapshot_cid,
            restored_count,
        });
    }

    #[test]
    fn test_broken_chain_detection() {
        // Arrange
        let mut store = MockCompositionEventStore::new();

        // Add valid events
        store.append_event(CompositionDomainEvent::GraphCreated {
            graph_id: "graph-1".to_string(),
            composition_type: "Functor".to_string(),
            root_node_id: "func-1".to_string(),
            timestamp: SystemTime::now(),
        }).unwrap();

        store.append_event(CompositionDomainEvent::NodeAdded {
            graph_id: "graph-1".to_string(),
            node_id: "transform-1".to_string(),
            node_type: "Transform".to_string(),
            label: "Map".to_string(),
            timestamp: SystemTime::now(),
        }).unwrap();

        // Manually break the chain
        if let Some(event) = store.events.get_mut(1) {
            event.previous_cid = Some(Cid::new(b"broken"));
        }

        // Act
        let result = store.validate_chain();

        // Assert
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Chain broken"));
    }

    #[test]
    fn test_graph_composition_event() {
        // Arrange
        let mut store = MockCompositionEventStore::new();

        // Act
        let event = CompositionDomainEvent::GraphComposed {
            graph_id: "composed-graph".to_string(),
            source_graph_id: "graph-a".to_string(),
            target_graph_id: "graph-b".to_string(),
            composition_type: "Sequential".to_string(),
            timestamp: SystemTime::now(),
        };

        let (event_id, cid, _) = store.append_event(event.clone()).unwrap();

        // Assert
        assert_eq!(store.events.len(), 1);
        match &store.events[0].event {
            CompositionDomainEvent::GraphComposed { composition_type, .. } => {
                assert_eq!(composition_type, "Sequential");
            }
            _ => panic!("Wrong event type"),
        }
        assert_eq!(store.events[0].event_id, event_id);
        assert_eq!(store.events[0].cid, cid);
    }
} 