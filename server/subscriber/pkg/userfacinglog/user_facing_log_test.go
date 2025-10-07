package userfacinglog

import (
	"testing"
	"time"
)

func TestNewUserFacingLogEvent(t *testing.T) {
	tests := []struct {
		name       string
		workflowID string
		jobID      string
		timestamp  time.Time
		level      UserFacingLogLevel
		nodeName   *string
		nodeID     *string
		message    string
		wantErr    bool
	}{
		{
			name:       "Valid event with all fields",
			workflowID: "workflow-123",
			jobID:      "job-456",
			timestamp:  time.Now(),
			level:      UserFacingLogLevelInfo,
			nodeName:   stringPtr("node-name"),
			nodeID:     stringPtr("node-id"),
			message:    "Test message",
			wantErr:    false,
		},
		{
			name:       "Valid event without optional fields",
			workflowID: "workflow-123",
			jobID:      "job-456",
			timestamp:  time.Now(),
			level:      UserFacingLogLevelSuccess,
			nodeName:   nil,
			nodeID:     nil,
			message:    "Success message",
			wantErr:    false,
		},
		{
			name:       "Invalid event - missing workflowID",
			workflowID: "",
			jobID:      "job-456",
			timestamp:  time.Now(),
			level:      UserFacingLogLevelError,
			nodeName:   nil,
			nodeID:     nil,
			message:    "Error message",
			wantErr:    true,
		},
		{
			name:       "Invalid event - missing jobID",
			workflowID: "workflow-123",
			jobID:      "",
			timestamp:  time.Now(),
			level:      UserFacingLogLevelError,
			nodeName:   nil,
			nodeID:     nil,
			message:    "Error message",
			wantErr:    true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			event, err := NewUserFacingLogEvent(
				tt.workflowID,
				tt.jobID,
				tt.timestamp,
				tt.level,
				tt.nodeName,
				tt.nodeID,
				tt.message,
			)

			if (err != nil) != tt.wantErr {
				t.Errorf("NewUserFacingLogEvent() error = %v, wantErr %v", err, tt.wantErr)
				return
			}

			if !tt.wantErr && event != nil {
				if event.WorkflowID != tt.workflowID {
					t.Errorf("WorkflowID = %v, want %v", event.WorkflowID, tt.workflowID)
				}
				if event.JobID != tt.jobID {
					t.Errorf("JobID = %v, want %v", event.JobID, tt.jobID)
				}
				if event.Level != tt.level {
					t.Errorf("Level = %v, want %v", event.Level, tt.level)
				}
				if event.Message != tt.message {
					t.Errorf("Message = %v, want %v", event.Message, tt.message)
				}
			}
		})
	}
}

func stringPtr(s string) *string {
	return &s
}
