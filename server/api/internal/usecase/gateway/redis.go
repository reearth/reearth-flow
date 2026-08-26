package gateway

import (
	"context"
	"time"

	"github.com/reearth/reearth-flow/api/pkg/diagnostic"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/log"
	"github.com/reearth/reearth-flow/api/pkg/userfacinglog"
)

type JobCompleteEvent struct {
	Timestamp             time.Time        `json:"timestamp"`
	DroppedEventCount     *uint64          `json:"droppedEventCount,omitempty"`
	WorkflowID            string           `json:"workflowId"`
	JobID                 string           `json:"jobId"`
	Result                string           `json:"result"`
	FailedNodes           []WireDiagnostic `json:"failedNodes,omitempty"`
	AggregatedDiagnostics []WireDiagnostic `json:"aggregatedDiagnostics,omitempty"`
}

type Redis interface {
	GetLogs(ctx context.Context, since time.Time, until time.Time, jobID id.JobID) ([]*log.Log, error)
	GetUserFacingLogs(ctx context.Context, since time.Time, until time.Time, jobID id.JobID) ([]*userfacinglog.UserFacingLog, error)
	GetJobCompleteEvent(ctx context.Context, jobID id.JobID) (*JobCompleteEvent, error)
	DeleteJobCompleteEvent(ctx context.Context, jobID id.JobID) error
	// Both return (nil, nil) for a missing key rather than an error.
	GetNodeDiagnostics(ctx context.Context, jobID id.JobID, nodeID string) ([]*diagnostic.Diagnostic, error)
	GetJobDiagnostics(ctx context.Context, jobID id.JobID) ([]*diagnostic.Diagnostic, error)
}
