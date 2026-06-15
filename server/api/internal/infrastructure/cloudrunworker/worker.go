package cloudrunworker

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"time"

	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"google.golang.org/api/idtoken"
)

// Worker implements gateway.CloudRunWorker by POSTing to a Cloud Run Service
// and blocking until the workflow finishes.
type Worker struct {
	file       gateway.File
	httpClient *http.Client
	serviceURL string
}

// New constructs a Worker using the provided file gateway for cancel-flag I/O.
func New(ctx context.Context, serviceURL string, file gateway.File) (gateway.CloudRunWorker, error) {
	httpClient, err := idtoken.NewClient(ctx, serviceURL)
	if err != nil {
		return nil, fmt.Errorf("cloudrunworker: idtoken client: %w", err)
	}
	// Bound the blocking /run call just above Cloud Run's max request timeout (60 min).
	httpClient.Timeout = 65 * time.Minute
	return &Worker{
		file:       file,
		httpClient: httpClient,
		serviceURL: serviceURL,
	}, nil
}

type runRequest struct {
	JobID         string            `json:"job_id"`
	WorkflowURL   string            `json:"workflow_url"`
	MetadataPath  string            `json:"metadata_path"`
	Variables     map[string]string `json:"variables,omitempty"`
	PreviousJobID *string           `json:"previous_job_id,omitempty"`
	StartNodeID   *string           `json:"start_node_id,omitempty"`
	CancelFlagURI string            `json:"cancel_flag_uri"`
}

type runResponse struct {
	Status string `json:"status"`
	Error  string `json:"error,omitempty"`
}

// probeSchemaRequest is the body for the dedicated /probe-schema route. It is
// deliberately separate from runRequest: probe-schema is a distinct worker
// subcommand and must not be expressed as a flag on /run.
type probeSchemaRequest struct {
	Variables   map[string]string `json:"variables,omitempty"`
	SampleSize  *int              `json:"sample_size,omitempty"`
	JobID       string            `json:"job_id"`
	WorkflowURL string            `json:"workflow_url"`
}

// RunJob POSTs to the Cloud Run Service /run endpoint and blocks until the
// workflow finishes. The service responds with a terminal status in the body.
func (w *Worker) RunJob(ctx context.Context, p gateway.RunJobParam) (gateway.JobStatus, error) {
	body := runRequest{
		JobID:         p.JobID.String(),
		WorkflowURL:   p.WorkflowURL,
		MetadataPath:  p.MetadataURL,
		Variables:     p.Variables,
		CancelFlagURI: w.file.CancelFlagURI(p.JobID.String()),
	}
	if p.PreviousJobID != nil {
		s := p.PreviousJobID.String()
		body.PreviousJobID = &s
	}
	if p.StartNodeID != nil {
		s := p.StartNodeID.String()
		body.StartNodeID = &s
	}

	buf, _ := json.Marshal(body)
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, w.serviceURL+"/run", bytes.NewReader(buf))
	if err != nil {
		return gateway.JobStatusFailed, err
	}
	req.Header.Set("Content-Type", "application/json")

	resp, err := w.httpClient.Do(req)
	if err != nil {
		// Connection dropped / instance reclaimed = infra failure.
		return gateway.JobStatusFailed, err
	}
	defer resp.Body.Close()

	raw, _ := io.ReadAll(resp.Body)
	var rr runResponse
	_ = json.Unmarshal(raw, &rr)

	switch rr.Status {
	case "COMPLETED":
		return gateway.JobStatusCompleted, nil
	case "CANCELLED":
		return gateway.JobStatusCancelled, nil
	default:
		detail := rr.Error
		if detail == "" {
			if len(raw) > 256 {
				detail = string(raw[:256])
			} else {
				detail = string(raw)
			}
		}
		return gateway.JobStatusFailed, fmt.Errorf(
			"cloudrunworker: run failed (http %d): %s", resp.StatusCode, detail,
		)
	}
}

// PreviewSchema POSTs to the Cloud Run Service /probe-schema endpoint and blocks
// until the probe finishes. The service runs `reearth-flow-worker probe-schema`,
// writes the SchemaReport JSON to the job's GCS output path, and responds with a
// terminal status in the body. This intentionally mirrors RunJob's auth/error
// handling but targets a dedicated route, not the /run path.
func (w *Worker) PreviewSchema(ctx context.Context, p gateway.ProbeSchemaParam) (gateway.JobStatus, error) {
	body := probeSchemaRequest{
		JobID:       p.JobID.String(),
		WorkflowURL: p.WorkflowURL,
		Variables:   p.Variables,
		SampleSize:  p.SampleSize,
	}

	buf, _ := json.Marshal(body)
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, w.serviceURL+"/probe-schema", bytes.NewReader(buf))
	if err != nil {
		return gateway.JobStatusFailed, err
	}
	req.Header.Set("Content-Type", "application/json")

	resp, err := w.httpClient.Do(req)
	if err != nil {
		// Connection dropped / instance reclaimed = infra failure.
		return gateway.JobStatusFailed, err
	}
	defer resp.Body.Close()

	raw, _ := io.ReadAll(resp.Body)
	var rr runResponse
	_ = json.Unmarshal(raw, &rr)

	switch rr.Status {
	case "COMPLETED":
		return gateway.JobStatusCompleted, nil
	case "CANCELLED":
		return gateway.JobStatusCancelled, nil
	default:
		detail := rr.Error
		if detail == "" {
			if len(raw) > 256 {
				detail = string(raw[:256])
			} else {
				detail = string(raw)
			}
		}
		return gateway.JobStatusFailed, fmt.Errorf(
			"cloudrunworker: probe-schema failed (http %d): %s", resp.StatusCode, detail,
		)
	}
}

// CancelJob writes a cancel flag via the file gateway so the wrapper process
// can trigger graceful cancellation of the running workflow.
func (w *Worker) CancelJob(ctx context.Context, jobID id.JobID) error {
	return w.file.WriteCancelFlag(ctx, jobID.String())
}
