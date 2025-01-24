package log

import (
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/pkg/id"
)

func TestNewLog(t *testing.T) {
	workflowID := WorkflowID(id.NewWorkflowID())
	jobID := JobID(id.NewJobID())
	nodeID := NodeID(id.NewNodeID())
	timestamp := time.Now()
	level := LevelInfo
	message := "Test log message"

	log := NewLog(workflowID, jobID, &nodeID, timestamp, level, message)

	if log.WorkflowID() != workflowID {
		t.Errorf("Expected WorkflowID %v, got %v", workflowID, log.WorkflowID())
	}
	if log.JobID() != jobID {
		t.Errorf("Expected JobID %v, got %v", jobID, log.JobID())
	}
	if log.NodeID() == nil || *log.NodeID() != nodeID {
		t.Errorf("Expected NodeID %v, got %v", nodeID, log.NodeID())
	}
	if !log.Timestamp().Equal(timestamp) {
		t.Errorf("Expected Timestamp %v, got %v", timestamp, log.Timestamp())
	}
	if log.Level() != level {
		t.Errorf("Expected Level %v, got %v", level, log.Level())
	}
	if log.Message() != message {
		t.Errorf("Expected Message %v, got %v", message, log.Message())
	}
}

func TestLogMethods(t *testing.T) {
	nodeID := NodeID(id.NewNodeID())
	workflowID := id.NewWorkflowID()
	jobID := id.NewJobID()
	log := NewLog(workflowID, jobID, &nodeID, time.Now(), LevelDebug, "Another test message")

	if log.WorkflowID() != workflowID {
		t.Errorf("Expected WorkflowID %v, got %v", workflowID, log.WorkflowID())
	}
	if log.JobID() != jobID {
		t.Errorf("Expected JobID %v, got %v", jobID, log.JobID())
	}
	if log.NodeID() == nil || *log.NodeID() != nodeID {
		t.Errorf("Expected NodeID %v, got %v", nodeID, log.NodeID())
	}
	if log.Level() != LevelDebug {
		t.Errorf("Expected Level 'DEBUG', got %v", log.Level())
	}
	if log.Message() != "Another test message" {
		t.Errorf("Expected Message 'Another test message', got %v", log.Message())
	}
}
