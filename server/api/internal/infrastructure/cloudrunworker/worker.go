package cloudrunworker

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"sync"
	"time"

	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"google.golang.org/api/idtoken"
)

// Worker implements gateway.CloudRunWorker by POSTing to a Cloud Run Service
// and blocking until the workflow finishes.
type Worker struct {
	// The cached answer to EngineVersion, valid until versionUntil. The lock is
	// held across the request, so concurrent callers share one.
	versionUntil time.Time
	file         gateway.File
	versionErr   error
	httpClient   *http.Client
	// now is the clock the version cache reads; nil is time.Now.
	now        func() time.Time
	serviceURL string
	version    string
	// versionLimit bounds one version request; zero is defaultVersionLimit.
	versionLimit time.Duration
	versionMu    sync.Mutex
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
	ReportURL   string            `json:"report_url"`
}

// renderViewRequest is the body for the dedicated /render-view route. Flat like
// probeSchemaRequest, and separate from it for the same reason: render-view is
// its own worker subcommand and must not be expressed as a flag on another.
//
// Both option groups are always sent; see featureview.Options for why.
type renderViewRequest struct {
	Row            *int    `json:"row,omitempty"`
	Filter         *string `json:"filter,omitempty"`
	TilesURL       *string `json:"tiles_url,omitempty"` // 2D
	Name           string  `json:"name"`
	InputURI       string  `json:"input_uri"`
	Shape          string  `json:"shape"`
	ReportURL      string  `json:"report_url"`
	OutputURI      string  `json:"output_uri"`
	TextureCodec   string  `json:"texture_codec"`    // 3D
	TexelSize      float64 `json:"texel_size"`       // 3D
	TargetTileSize uint64  `json:"target_tile_size"` // 3D
	MaxTileBytes   uint64  `json:"max_tile_bytes"`   // 2D
	Extent         int32   `json:"extent"`           // 2D
	Draco          bool    `json:"draco"`            // 3D
	MinZoom        uint8   `json:"min_zoom"`         // 2D
	MaxZoom        uint8   `json:"max_zoom"`         // 2D
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
		ReportURL:   p.ReportURL,
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

// RenderView POSTs to the Cloud Run Service /render-view endpoint and blocks
// until the render finishes. The service runs `reearth-flow-worker
// render-view`, writes the view and its report to the job's GCS artifact
// prefix, and responds with a terminal status.
//
// A render that drew nothing is NOT a failure here: the renderer records
// "empty" or "unsupported geometry" in the report and exits successfully, so
// the reason reaches the user as a status instead of an exit code. Only a real
// fault comes back as JobStatusFailed.
func (w *Worker) RenderView(ctx context.Context, p gateway.RenderViewParam) (gateway.JobStatus, error) {
	body := renderViewRequest{
		InputURI:  p.InputURI,
		OutputURI: p.OutputURI,
		ReportURL: p.ReportURI,
		Name:      p.Name,
		Shape:     string(p.Shape),
		Row:       p.Row,
		Filter:    p.Filter,
		TilesURL:  p.TilesURL,

		Draco:          p.Options.Draco,
		TexelSize:      p.Options.TexelSize,
		TextureCodec:   string(p.Options.TextureCodec),
		TargetTileSize: p.Options.TargetTileSize,

		MinZoom:      p.Options.MinZoom,
		MaxZoom:      p.Options.MaxZoom,
		Extent:       p.Options.Extent,
		MaxTileBytes: p.Options.MaxTileBytes,
	}

	buf, _ := json.Marshal(body)
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, w.serviceURL+"/render-view", bytes.NewReader(buf))
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
			"cloudrunworker: render-view failed (http %d): %s", resp.StatusCode, detail,
		)
	}
}

// CancelJob writes a cancel flag via the file gateway so the wrapper process
// can trigger graceful cancellation of the running workflow.
func (w *Worker) CancelJob(ctx context.Context, jobID id.JobID) error {
	return w.file.WriteCancelFlag(ctx, jobID.String())
}

const (
	// defaultVersionLimit bounds a version request. The client's own timeout
	// is sized for an hour-long run, far too long for this.
	defaultVersionLimit = 10 * time.Second
	// versionTTL and versionFailureTTL are how long an answer is kept. A
	// failure is retried sooner: a 404 is expected until the worker is
	// redeployed with the route, and it should be noticed soon after.
	versionTTL        = 5 * time.Minute
	versionFailureTTL = time.Minute
)

// EngineVersion asks the Service which engine it runs, at `GET /version`.
//
// The answer is cached. The Service takes one request per instance and is
// given CPU only while a request is open, so asking on every render could
// start an instance just to answer. A success is kept for five minutes and a
// failure for one.
func (w *Worker) EngineVersion(ctx context.Context) (string, error) {
	w.versionMu.Lock()
	defer w.versionMu.Unlock()

	if w.clock().Before(w.versionUntil) {
		return w.version, w.versionErr
	}
	v, err := w.fetchEngineVersion(ctx)
	ttl := versionTTL
	if err != nil {
		ttl = versionFailureTTL
	}
	w.version, w.versionErr, w.versionUntil = v, err, w.clock().Add(ttl)
	return v, err
}

func (w *Worker) fetchEngineVersion(ctx context.Context) (string, error) {
	// The answer is shared by every caller, so one caller giving up must not
	// be remembered as the Service failing.
	limit := w.versionLimit
	if limit == 0 {
		limit = defaultVersionLimit
	}
	ctx, cancel := context.WithTimeout(context.WithoutCancel(ctx), limit)
	defer cancel()

	req, err := http.NewRequestWithContext(ctx, http.MethodGet, w.serviceURL+"/version", nil)
	if err != nil {
		return "", err
	}
	resp, err := w.httpClient.Do(req)
	if err != nil {
		return "", fmt.Errorf("cloudrunworker: version: %w", err)
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		return "", fmt.Errorf("cloudrunworker: version: http %d", resp.StatusCode)
	}
	var body struct {
		EngineVersion string `json:"engineVersion"`
	}
	if err := json.NewDecoder(io.LimitReader(resp.Body, 4096)).Decode(&body); err != nil {
		return "", fmt.Errorf("cloudrunworker: version: %w", err)
	}
	if body.EngineVersion == "" {
		return "", errors.New("cloudrunworker: version: the Service named no engine version")
	}
	return body.EngineVersion, nil
}

func (w *Worker) clock() time.Time {
	if w.now == nil {
		return time.Now()
	}
	return w.now()
}
