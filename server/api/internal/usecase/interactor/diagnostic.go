package interactor

import (
	"context"
	"encoding/json"
	"errors"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"

	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/diagnostic"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/log"
	"github.com/reearth/reearthx/rerror"
)

// Reads hit both Redis and Mongo: stopping at a non-empty Redis result would hide terminal rows.
// The diagnostics.json the worker uploads with the job's artifacts is the durable
// fallback when the database has no rows (e.g. a driver without a diagnostics repo).
type NodeDiagnostics struct {
	diagnosticsRepo   repo.NodeDiagnostics
	jobRepo           repo.Job
	redisGateway      gateway.Redis
	file              gateway.File
	permissionChecker gateway.PermissionChecker
}

func NewNodeDiagnostics(diagnosticsRepo repo.NodeDiagnostics, jobRepo repo.Job, redisGateway gateway.Redis, file gateway.File, permissionChecker gateway.PermissionChecker) interfaces.NodeDiagnostics {
	return &NodeDiagnostics{
		diagnosticsRepo:   diagnosticsRepo,
		jobRepo:           jobRepo,
		redisGateway:      redisGateway,
		file:              file,
		permissionChecker: permissionChecker,
	}
}

// readArtifactDiagnostics loads the worker-written diagnostics artifact. Absent
// or unreadable files degrade to nil: the artifact only exists for finished
// runs, and a fallback source must never fail the query.
func (i *NodeDiagnostics) readArtifactDiagnostics(ctx context.Context, jobID id.JobID) *gateway.JobCompleteEvent {
	if i.file == nil {
		return nil
	}
	rc, err := i.file.ReadArtifact(ctx, jobID.String()+"/diagnostics.json")
	if err != nil || rc == nil {
		if err != nil && !errors.Is(err, rerror.ErrNotFound) {
			log.Warnfc(ctx, "diagnostic: failed to read diagnostics artifact: %v", err)
		}
		return nil
	}
	defer func() { _ = rc.Close() }()

	var event gateway.JobCompleteEvent
	if err := json.NewDecoder(rc).Decode(&event); err != nil {
		log.Warnfc(ctx, "diagnostic: failed to decode diagnostics artifact: %v", err)
		return nil
	}
	return &event
}

func (i *NodeDiagnostics) artifactRows(ctx context.Context, jobID id.JobID) []*diagnostic.Diagnostic {
	event := i.readArtifactDiagnostics(ctx, jobID)
	if event == nil {
		return nil
	}
	wire := make([]gateway.WireDiagnostic, 0, len(event.FailedNodes)+len(event.AggregatedDiagnostics))
	wire = append(wire, event.FailedNodes...)
	wire = append(wire, event.AggregatedDiagnostics...)
	rows, err := wireDiagnosticsToDomain(jobID, event.Timestamp, wire)
	if err != nil {
		log.Warnfc(ctx, "diagnostic: failed to convert diagnostics artifact rows: %v", err)
		return nil
	}
	return rows
}

func (i *NodeDiagnostics) checkPermission(ctx context.Context, action string, workspaceID ...accountsid.WorkspaceID) error {
	return checkPermission(ctx, i.permissionChecker, rbac.ResourceJob, action, workspaceID...)
}

func (i *NodeDiagnostics) checkJobPermission(ctx context.Context, jobID id.JobID) error {
	j, err := i.jobRepo.FindByID(ctx, jobID)
	if err != nil {
		return err
	}
	var wsIDs []accountsid.WorkspaceID
	if j != nil {
		wsIDs = append(wsIDs, j.Workspace())
	}
	return i.checkPermission(ctx, rbac.ActionAny, wsIDs...)
}

func (i *NodeDiagnostics) GetNodeDiagnostics(ctx context.Context, jobID id.JobID, nodeID string) ([]*diagnostic.Diagnostic, error) {
	if err := i.checkJobPermission(ctx, jobID); err != nil {
		return nil, err
	}

	var rows []*diagnostic.Diagnostic

	if i.redisGateway != nil {
		liveRows, err := i.redisGateway.GetNodeDiagnostics(ctx, jobID, nodeID)
		if err != nil {
			log.Warnfc(ctx, "diagnostic: failed to get node diagnostics from Redis: %v", err)
		} else {
			rows = append(rows, liveRows...)
		}
	}

	durable := 0
	if i.diagnosticsRepo != nil {
		mongoRows, err := i.diagnosticsRepo.FindByJobNodeID(ctx, jobID, nodeID)
		if err != nil {
			return nil, err
		}
		durable = len(mongoRows)
		rows = append(rows, mongoRows...)
	}
	if durable == 0 {
		for _, row := range i.artifactRows(ctx, jobID) {
			if row.NodeID() != nil && *row.NodeID() == nodeID {
				rows = append(rows, row)
			}
		}
	}

	return dedupeDiagnostics(rows), nil
}

func (i *NodeDiagnostics) GetJobDiagnostics(ctx context.Context, jobID id.JobID) ([]*diagnostic.Diagnostic, error) {
	if err := i.checkJobPermission(ctx, jobID); err != nil {
		return nil, err
	}

	var rows []*diagnostic.Diagnostic

	if i.redisGateway != nil {
		liveRows, err := i.redisGateway.GetJobDiagnostics(ctx, jobID)
		if err != nil {
			log.Warnfc(ctx, "diagnostic: failed to get job diagnostics from Redis: %v", err)
		} else {
			rows = append(rows, liveRows...)
		}
	}

	durable := 0
	if i.diagnosticsRepo != nil {
		mongoRows, err := i.diagnosticsRepo.FindByJobID(ctx, jobID)
		if err != nil {
			return nil, err
		}
		durable = len(mongoRows)
		rows = append(rows, mongoRows...)
	}
	if durable == 0 {
		rows = append(rows, i.artifactRows(ctx, jobID)...)
	}

	return dedupeDiagnostics(rows), nil
}

// Stamped on failedNodes rows and never on aggregated ones, which is how the two are told apart.
const fatalEffectiveDisposition = "fatal"

// Terminal-only: rows are written just once, at job-completion merge, to the
// database and to the diagnostics artifact.
func (i *NodeDiagnostics) GetFailedNodes(ctx context.Context, jobID id.JobID) ([]*diagnostic.Diagnostic, error) {
	if err := i.checkJobPermission(ctx, jobID); err != nil {
		return nil, err
	}

	var rows []*diagnostic.Diagnostic
	if i.diagnosticsRepo != nil {
		var err error
		rows, err = i.diagnosticsRepo.FindByJobID(ctx, jobID)
		if err != nil {
			return nil, err
		}
	}

	failed := filterFatal(rows)
	if len(failed) == 0 {
		failed = filterFatal(i.artifactRows(ctx, jobID))
	}
	return dedupeDiagnostics(failed), nil
}

func filterFatal(rows []*diagnostic.Diagnostic) []*diagnostic.Diagnostic {
	failed := make([]*diagnostic.Diagnostic, 0, len(rows))
	for _, row := range rows {
		if ed := row.EffectiveDisposition(); ed != nil && *ed == fatalEffectiveDisposition {
			failed = append(failed, row)
		}
	}
	return failed
}

// disposition is in the key because a failed and an aggregated row can share (nodeID, code).
func dedupeDiagnostics(rows []*diagnostic.Diagnostic) []*diagnostic.Diagnostic {
	type dedupeKey struct {
		nodeID      string
		code        string
		disposition string
	}

	keyOf := func(d *diagnostic.Diagnostic) dedupeKey {
		nodeID := ""
		if d.NodeID() != nil {
			nodeID = *d.NodeID()
		}
		disposition := ""
		if d.EffectiveDisposition() != nil {
			disposition = *d.EffectiveDisposition()
		}
		return dedupeKey{nodeID: nodeID, code: d.Code(), disposition: disposition}
	}

	order := make([]dedupeKey, 0, len(rows))
	best := make(map[dedupeKey]*diagnostic.Diagnostic, len(rows))
	for _, row := range rows {
		if row == nil {
			continue
		}
		k := keyOf(row)
		existing, ok := best[k]
		if !ok {
			order = append(order, k)
			best[k] = row
			continue
		}
		if preferOver(row, existing) {
			best[k] = row
		}
	}

	out := make([]*diagnostic.Diagnostic, 0, len(order))
	for _, k := range order {
		out = append(out, best[k])
	}
	return out
}

func preferOver(candidate, current *diagnostic.Diagnostic) bool {
	if candidate.Terminal() != current.Terminal() {
		return candidate.Terminal()
	}
	return candidate.Timestamp().After(current.Timestamp())
}

func (i *NodeDiagnostics) GetDroppedEventCount(ctx context.Context, jobID id.JobID) (*uint64, error) {
	if err := i.checkJobPermission(ctx, jobID); err != nil {
		return nil, err
	}

	if i.diagnosticsRepo != nil {
		count, err := i.diagnosticsRepo.FindJobSummary(ctx, jobID)
		if err != nil {
			return nil, err
		}
		if count != nil {
			return count, nil
		}
	}

	if event := i.readArtifactDiagnostics(ctx, jobID); event != nil {
		return event.DroppedEventCount, nil
	}
	return nil, nil
}
