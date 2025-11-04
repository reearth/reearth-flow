package job

import (
	"testing"

	"github.com/reearth/reearth-flow/api/pkg/id"
)

func TestJob_StatusComputation(t *testing.T) {
	tests := []struct {
		name         string
		batchStatus  *Status
		workerStatus *Status
		legacyStatus Status
		expected     Status
		description  string
	}{
		{
			name:         "Both nil - uses legacy status",
			batchStatus:  nil,
			workerStatus: nil,
			legacyStatus: StatusCompleted,
			expected:     StatusCompleted,
			description:  "Backward compatibility: old jobs without new fields",
		},
		{
			name:         "Only batch status available",
			batchStatus:  ptr(StatusRunning),
			workerStatus: nil,
			legacyStatus: StatusPending,
			expected:     StatusRunning,
			description:  "Worker not reported yet",
		},
		{
			name:         "Only worker status available",
			batchStatus:  nil,
			workerStatus: ptr(StatusCompleted),
			legacyStatus: StatusPending,
			expected:     StatusCompleted,
			description:  "Rare case: worker status without batch",
		},
		{
			name:         "Both succeeded",
			batchStatus:  ptr(StatusCompleted),
			workerStatus: ptr(StatusCompleted),
			legacyStatus: StatusPending,
			expected:     StatusCompleted,
			description:  "Both batch and worker completed successfully",
		},
		{
			name:         "Batch succeeded, worker failed - CRITICAL BUG FIX",
			batchStatus:  ptr(StatusCompleted),
			workerStatus: ptr(StatusFailed),
			legacyStatus: StatusPending,
			expected:     StatusFailed,
			description:  "Container ran but workflow execution failed",
		},
		{
			name:         "Batch failed, worker succeeded",
			batchStatus:  ptr(StatusFailed),
			workerStatus: ptr(StatusCompleted),
			legacyStatus: StatusPending,
			expected:     StatusFailed,
			description:  "Container failed even though worker reported success",
		},
		{
			name:         "Both failed",
			batchStatus:  ptr(StatusFailed),
			workerStatus: ptr(StatusFailed),
			legacyStatus: StatusPending,
			expected:     StatusFailed,
			description:  "Both batch and worker failed",
		},
		{
			name:         "Batch cancelled",
			batchStatus:  ptr(StatusCancelled),
			workerStatus: ptr(StatusRunning),
			legacyStatus: StatusPending,
			expected:     StatusCancelled,
			description:  "Cancellation takes precedence",
		},
		{
			name:         "Batch running, worker not started",
			batchStatus:  ptr(StatusRunning),
			workerStatus: nil,
			legacyStatus: StatusPending,
			expected:     StatusRunning,
			description:  "Job is running, worker hasn't reported",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			j := &Job{
				batchStatus:  tt.batchStatus,
				workerStatus: tt.workerStatus,
				status:       tt.legacyStatus,
			}

			got := j.Status()
			if got != tt.expected {
				t.Errorf("Status() = %v, want %v\nDescription: %s", got, tt.expected, tt.description)
			}
		})
	}
}

func TestJob_SetBatchStatus(t *testing.T) {
	j := &Job{}
	status := StatusRunning

	j.SetBatchStatus(status)

	if j.batchStatus == nil {
		t.Error("SetBatchStatus() should set batchStatus to non-nil")
	}
	if *j.batchStatus != status {
		t.Errorf("SetBatchStatus() = %v, want %v", *j.batchStatus, status)
	}
}

func TestJob_SetWorkerStatus(t *testing.T) {
	j := &Job{}
	status := StatusCompleted

	j.SetWorkerStatus(status)

	if j.workerStatus == nil {
		t.Error("SetWorkerStatus() should set workerStatus to non-nil")
	}
	if *j.workerStatus != status {
		t.Errorf("SetWorkerStatus() = %v, want %v", *j.workerStatus, status)
	}
}

func TestJob_BatchStatusGetter(t *testing.T) {
	status := StatusRunning
	j := &Job{batchStatus: &status}

	got := j.BatchStatus()
	if got == nil {
		t.Error("BatchStatus() should return non-nil")
	}
	if *got != status {
		t.Errorf("BatchStatus() = %v, want %v", *got, status)
	}

	// Test nil case
	j2 := &Job{}
	if j2.BatchStatus() != nil {
		t.Error("BatchStatus() should return nil when not set")
	}
}

func TestJob_WorkerStatusGetter(t *testing.T) {
	status := StatusCompleted
	j := &Job{workerStatus: &status}

	got := j.WorkerStatus()
	if got == nil {
		t.Error("WorkerStatus() should return non-nil")
	}
	if *got != status {
		t.Errorf("WorkerStatus() = %v, want %v", *got, status)
	}

	// Test nil case
	j2 := &Job{}
	if j2.WorkerStatus() != nil {
		t.Error("WorkerStatus() should return nil when not set")
	}
}

func TestNewJob_InitializesBatchStatus(t *testing.T) {
	jobID := NewID()
	deploymentID, _ := DeploymentIDFrom("01234567-89ab-cdef-0123-456789abcdef")
	workspaceID, _ := id.WorkspaceIDFrom("01234567-89ab-cdef-0123-456789abcde0")

	j := NewJob(jobID, deploymentID, workspaceID, "gcp-job-123")

	if j.batchStatus == nil {
		t.Error("NewJob() should initialize batchStatus to non-nil")
	}
	if *j.batchStatus != StatusPending {
		t.Errorf("NewJob() should initialize batchStatus to StatusPending, got %v", *j.batchStatus)
	}
	if j.workerStatus != nil {
		t.Error("NewJob() should initialize workerStatus to nil")
	}
}

// Helper function to create pointer to Status
func ptr(s Status) *Status {
	return &s
}
