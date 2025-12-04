#[cfg(test)]
mod tests {
    use crate::event::{Event, EventHub};
    use crate::node::{NodeHandle, NodeId, NodeStatus};
    use tracing::Level;
    use uuid::Uuid;

    #[test]
    fn test_event_hub_new() {
        let hub = EventHub::new(10);
        assert_eq!(hub.sender.receiver_count(), 1);
    }

    #[test]
    fn test_event_hub_clone() {
        let hub1 = EventHub::new(10);
        let _hub2 = hub1.clone();
        
        assert_eq!(hub1.sender.receiver_count(), 2);
    }

    #[test]
    fn test_event_hub_send_info_log() {
        let hub = EventHub::new(10);
        let mut receiver = hub.sender.subscribe();
        
        hub.info_log(None, "Test message");
        
        let event = receiver.try_recv().unwrap();
        match event {
            Event::Log { level, message, .. } => {
                assert_eq!(level, Level::INFO);
                assert_eq!(message, "Test message");
            }
            _ => panic!("Expected Log event"),
        }
    }

    #[test]
    fn test_event_hub_send_debug_log() {
        let hub = EventHub::new(10);
        let mut receiver = hub.sender.subscribe();
        
        hub.debug_log(None, "Debug message");
        
        let event = receiver.try_recv().unwrap();
        match event {
            Event::Log { level, message, .. } => {
                assert_eq!(level, Level::DEBUG);
                assert_eq!(message, "Debug message");
            }
            _ => panic!("Expected Log event"),
        }
    }

    #[test]
    fn test_event_hub_send_warn_log() {
        let hub = EventHub::new(10);
        let mut receiver = hub.sender.subscribe();
        
        hub.warn_log(None, "Warning message");
        
        let event = receiver.try_recv().unwrap();
        match event {
            Event::Log { level, message, .. } => {
                assert_eq!(level, Level::WARN);
                assert_eq!(message, "Warning message");
            }
            _ => panic!("Expected Log event"),
        }
    }

    #[test]
    fn test_event_hub_send_error_log() {
        let hub = EventHub::new(10);
        let mut receiver = hub.sender.subscribe();
        
        hub.error_log(None, "Error message");
        
        let event = receiver.try_recv().unwrap();
        match event {
            Event::Log { level, message, .. } => {
                assert_eq!(level, Level::ERROR);
                assert_eq!(message, "Error message");
            }
            _ => panic!("Expected Log event"),
        }
    }

    #[test]
    fn test_event_hub_info_log_with_node_handle() {
        let hub = EventHub::new(10);
        let mut receiver = hub.sender.subscribe();
        let node_handle = NodeHandle {
            id: NodeId::new("test_node"),
        };
        
        hub.info_log_with_node_handle(None, node_handle.clone(), "Node message");
        
        let event = receiver.try_recv().unwrap();
        match event {
            Event::Log { level, node_handle: nh, message, .. } => {
                assert_eq!(level, Level::INFO);
                assert_eq!(message, "Node message");
                assert_eq!(nh.unwrap().id, node_handle.id);
            }
            _ => panic!("Expected Log event"),
        }
    }

    #[test]
    fn test_event_hub_info_log_with_node_info() {
        let hub = EventHub::new(10);
        let mut receiver = hub.sender.subscribe();
        let node_handle = NodeHandle {
            id: NodeId::new("test_node"),
        };
        
        hub.info_log_with_node_info(None, node_handle.clone(), "NodeName".to_string(), "Message");
        
        let event = receiver.try_recv().unwrap();
        match event {
            Event::Log { level, node_handle: nh, node_name, message, .. } => {
                assert_eq!(level, Level::INFO);
                assert_eq!(message, "Message");
                assert_eq!(nh.unwrap().id, node_handle.id);
                assert_eq!(node_name, Some("NodeName".to_string()));
            }
            _ => panic!("Expected Log event"),
        }
    }

    #[test]
    fn test_event_hub_debug_log_with_node_handle() {
        let hub = EventHub::new(10);
        let mut receiver = hub.sender.subscribe();
        let node_handle = NodeHandle {
            id: NodeId::new("test_node"),
        };
        
        hub.debug_log_with_node_handle(None, node_handle.clone(), "Debug node message");
        
        let event = receiver.try_recv().unwrap();
        match event {
            Event::Log { level, message, .. } => {
                assert_eq!(level, Level::DEBUG);
                assert_eq!(message, "Debug node message");
            }
            _ => panic!("Expected Log event"),
        }
    }

    #[test]
    fn test_event_hub_warn_log_with_node_handle() {
        let hub = EventHub::new(10);
        let mut receiver = hub.sender.subscribe();
        let node_handle = NodeHandle {
            id: NodeId::new("test_node"),
        };
        
        hub.warn_log_with_node_handle(None, node_handle, "Warning node message");
        
        let event = receiver.try_recv().unwrap();
        match event {
            Event::Log { level, message, .. } => {
                assert_eq!(level, Level::WARN);
                assert_eq!(message, "Warning node message");
            }
            _ => panic!("Expected Log event"),
        }
    }

    #[test]
    fn test_event_hub_error_log_with_node_handle() {
        let hub = EventHub::new(10);
        let mut receiver = hub.sender.subscribe();
        let node_handle = NodeHandle {
            id: NodeId::new("test_node"),
        };
        
        hub.error_log_with_node_handle(None, node_handle, "Error node message");
        
        let event = receiver.try_recv().unwrap();
        match event {
            Event::Log { level, message, .. } => {
                assert_eq!(level, Level::ERROR);
                assert_eq!(message, "Error node message");
            }
            _ => panic!("Expected Log event"),
        }
    }

    #[test]
    fn test_event_hub_error_log_with_node_info() {
        let hub = EventHub::new(10);
        let mut receiver = hub.sender.subscribe();
        let node_handle = NodeHandle {
            id: NodeId::new("test_node"),
        };
        
        hub.error_log_with_node_info(None, node_handle.clone(), "NodeName".to_string(), "Error");
        
        let event = receiver.try_recv().unwrap();
        match event {
            Event::Log { level, node_name, message, .. } => {
                assert_eq!(level, Level::ERROR);
                assert_eq!(message, "Error");
                assert_eq!(node_name, Some("NodeName".to_string()));
            }
            _ => panic!("Expected Log event"),
        }
    }

    #[test]
    fn test_event_source_flushed() {
        let hub = EventHub::new(10);
        let mut receiver = hub.sender.subscribe();
        
        hub.send(Event::SourceFlushed);
        
        let event = receiver.try_recv().unwrap();
        assert!(matches!(event, Event::SourceFlushed));
    }

    #[test]
    fn test_event_processor_finished() {
        let hub = EventHub::new(10);
        let mut receiver = hub.sender.subscribe();
        let node_handle = NodeHandle {
            id: NodeId::new("proc1"),
        };
        
        hub.send(Event::ProcessorFinished {
            node: node_handle.clone(),
            name: "TestProcessor".to_string(),
        });
        
        let event = receiver.try_recv().unwrap();
        match event {
            Event::ProcessorFinished { node, name } => {
                assert_eq!(node.id, node_handle.id);
                assert_eq!(name, "TestProcessor");
            }
            _ => panic!("Expected ProcessorFinished event"),
        }
    }

    #[test]
    fn test_event_processor_failed() {
        let hub = EventHub::new(10);
        let mut receiver = hub.sender.subscribe();
        let node_handle = NodeHandle {
            id: NodeId::new("proc1"),
        };
        
        hub.send(Event::ProcessorFailed {
            node: node_handle.clone(),
            name: "TestProcessor".to_string(),
        });
        
        let event = receiver.try_recv().unwrap();
        match event {
            Event::ProcessorFailed { node, name } => {
                assert_eq!(node.id, node_handle.id);
                assert_eq!(name, "TestProcessor");
            }
            _ => panic!("Expected ProcessorFailed event"),
        }
    }

    #[test]
    fn test_event_sink_finished() {
        let hub = EventHub::new(10);
        let mut receiver = hub.sender.subscribe();
        let node_handle = NodeHandle {
            id: NodeId::new("sink1"),
        };
        
        hub.send(Event::SinkFinished {
            node: node_handle.clone(),
            name: "TestSink".to_string(),
        });
        
        let event = receiver.try_recv().unwrap();
        match event {
            Event::SinkFinished { node, name } => {
                assert_eq!(node.id, node_handle.id);
                assert_eq!(name, "TestSink");
            }
            _ => panic!("Expected SinkFinished event"),
        }
    }

    #[test]
    fn test_event_sink_finish_failed() {
        let hub = EventHub::new(10);
        let mut receiver = hub.sender.subscribe();
        
        hub.send(Event::SinkFinishFailed {
            name: "FailedSink".to_string(),
        });
        
        let event = receiver.try_recv().unwrap();
        match event {
            Event::SinkFinishFailed { name } => {
                assert_eq!(name, "FailedSink");
            }
            _ => panic!("Expected SinkFinishFailed event"),
        }
    }

    #[test]
    fn test_event_edge_completed() {
        let hub = EventHub::new(10);
        let mut receiver = hub.sender.subscribe();
        let feature_id = Uuid::new_v4();
        let edge_id = crate::node::EdgeId::new("edge1");
        
        hub.send(Event::EdgeCompleted {
            feature_id,
            edge_id: edge_id.clone(),
        });
        
        let event = receiver.try_recv().unwrap();
        match event {
            Event::EdgeCompleted { feature_id: fid, edge_id: eid } => {
                assert_eq!(fid, feature_id);
                assert_eq!(eid, edge_id);
            }
            _ => panic!("Expected EdgeCompleted event"),
        }
    }

    #[test]
    fn test_event_edge_pass_through() {
        let hub = EventHub::new(10);
        let mut receiver = hub.sender.subscribe();
        let feature_id = Uuid::new_v4();
        let edge_id = crate::node::EdgeId::new("edge1");
        
        hub.send(Event::EdgePassThrough {
            feature_id,
            edge_id: edge_id.clone(),
        });
        
        let event = receiver.try_recv().unwrap();
        match event {
            Event::EdgePassThrough { feature_id: fid, edge_id: eid } => {
                assert_eq!(fid, feature_id);
                assert_eq!(eid, edge_id);
            }
            _ => panic!("Expected EdgePassThrough event"),
        }
    }

    #[test]
    fn test_event_node_status_changed() {
        let hub = EventHub::new(10);
        let mut receiver = hub.sender.subscribe();
        let node_handle = NodeHandle {
            id: NodeId::new("node1"),
        };
        let feature_id = Uuid::new_v4();
        
        hub.send(Event::NodeStatusChanged {
            node_handle: node_handle.clone(),
            status: NodeStatus::Starting,
            feature_id: Some(feature_id),
        });
        
        let event = receiver.try_recv().unwrap();
        match event {
            Event::NodeStatusChanged { node_handle: nh, status, feature_id: fid } => {
                assert_eq!(nh.id, node_handle.id);
                assert_eq!(status, NodeStatus::Starting);
                assert_eq!(fid, Some(feature_id));
            }
            _ => panic!("Expected NodeStatusChanged event"),
        }
    }

    #[tokio::test]
    async fn test_event_hub_simple_flush() {
        let hub = EventHub::new(10);
        hub.simple_flush(10).await;
    }

    #[tokio::test]
    async fn test_event_hub_enhanced_flush_no_receivers() {
        let hub = EventHub::new(10);
        {
            let _receiver = hub.sender.subscribe();
        }
        
        hub.enhanced_flush(100).await;
    }

    #[test]
    fn test_event_hub_multiple_subscribers() {
        let hub = EventHub::new(10);
        let mut receiver1 = hub.sender.subscribe();
        let mut receiver2 = hub.sender.subscribe();
        
        hub.send(Event::SourceFlushed);
        
        let event1 = receiver1.try_recv().unwrap();
        let event2 = receiver2.try_recv().unwrap();
        
        assert!(matches!(event1, Event::SourceFlushed));
        assert!(matches!(event2, Event::SourceFlushed));
    }

    #[test]
    fn test_event_hub_capacity() {
        let capacity = 5;
        let hub = EventHub::new(capacity);
        let mut receiver = hub.sender.subscribe();
        
        for _ in 0..capacity {
            hub.send(Event::SourceFlushed);
        }
        
        for _ in 0..capacity {
            assert!(receiver.try_recv().is_ok());
        }
        
        assert!(receiver.try_recv().is_err());
    }

    #[test]
    fn test_event_log_with_string_conversion() {
        let hub = EventHub::new(10);
        let mut receiver = hub.sender.subscribe();
        let number = 42;
        
        hub.info_log(None, format!("Number: {}", number));
        
        let event = receiver.try_recv().unwrap();
        match event {
            Event::Log { message, .. } => {
                assert_eq!(message, "Number: 42");
            }
            _ => panic!("Expected Log event"),
        }
    }
}

