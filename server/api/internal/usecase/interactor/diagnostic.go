package interactor

import (
	"context"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"

	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/diagnostic"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/log"
)

// NodeDiagnostics reads structured diagnostics for a job/node.
//
// Merges live Redis rows with durable Mongo rows (deduped via
// dedupeDiagnostics); always consults both, since Mongo alone holds
// job-completion terminal rows that never reach Redis — short-circuiting on
// a non-empty Redis result would hide those until the Redis TTL expired.
type NodeDiagnostics struct {
	diagnosticsRepo   repo.NodeDiagnostics
	jobRepo           repo.Job
	redisGateway      gateway.Redis
	permissionChecker gateway.PermissionChecker
}

func NewNodeDiagnostics(diagnosticsRepo repo.NodeDiagnostics, jobRepo repo.Job, redisGateway gateway.Redis, permissionChecker gateway.PermissionChecker) interfaces.NodeDiagnostics {
	return &NodeDiagnostics{
		diagnosticsRepo:   diagnosticsRepo,
		jobRepo:           jobRepo,
		redisGateway:      redisGateway,
		permissionChecker: permissionChecker,
	}
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

	if i.diagnosticsRepo != nil {
		mongoRows, err := i.diagnosticsRepo.FindByJobNodeID(ctx, jobID, nodeID)
		if err != nil {
			return nil, err
		}
		rows = append(rows, mongoRows...)
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

	if i.diagnosticsRepo != nil {
		mongoRows, err := i.diagnosticsRepo.FindByJobID(ctx, jobID)
		if err != nil {
			return nil, err
		}
		rows = append(rows, mongoRows...)
	}

	return dedupeDiagnostics(rows), nil
}

// fatalEffectiveDisposition is the wire value of the engine's
// Disposition::Fatal. failedNodes entries are always stamped Fatal;
// aggregatedDiagnostics entries never are — this recovers which wire array a
// persisted row came from (see GetFailedNodes).
const fatalEffectiveDisposition = "fatal"

// GetFailedNodes reads the job's terminal per-node fatal-failure rows
// (GraphQL Job.failedNodes). Deliberately Mongo-only, never Redis:
// failedNodes rows are persisted exclusively at job-completion merge time
// and never written to Redis. Filtered via fatalEffectiveDisposition, since
// only failedNodes rows carry that disposition.
func (i *NodeDiagnostics) GetFailedNodes(ctx context.Context, jobID id.JobID) ([]*diagnostic.Diagnostic, error) {
	if err := i.checkJobPermission(ctx, jobID); err != nil {
		return nil, err
	}

	if i.diagnosticsRepo == nil {
		return []*diagnostic.Diagnostic{}, nil
	}

	rows, err := i.diagnosticsRepo.FindByJobID(ctx, jobID)
	if err != nil {
		return nil, err
	}

	failed := make([]*diagnostic.Diagnostic, 0, len(rows))
	for _, row := range rows {
		if ed := row.EffectiveDisposition(); ed != nil && *ed == fatalEffectiveDisposition {
			failed = append(failed, row)
		}
	}
	return dedupeDiagnostics(failed), nil
}

// dedupeDiagnostics collapses rows sharing a (nodeId, code,
// effectiveDisposition) key to one representative (preferOver decides the
// winner), keeping first-occurrence order. Needed because an aggregated
// summary is legitimately persisted twice — once as a live mirror, once as
// a terminal row at job completion. effectiveDisposition is part of the key
// because a failed-node row and an aggregated row can otherwise share
// (nodeId, code) and would wrongly collapse into each other.
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

// preferOver reports whether candidate should replace current as
// dedupeDiagnostics' representative row: Terminal() wins ties, else the
// more recent Timestamp wins.
func preferOver(candidate, current *diagnostic.Diagnostic) bool {
	if candidate.Terminal() != current.Terminal() {
		return candidate.Terminal()
	}
	return candidate.Timestamp().After(current.Timestamp())
}

// GetDroppedEventCount reads the job's persisted droppedEventCount (GraphQL
// Job.droppedEventCount) from the per-job summary row written alongside
// failedNodes/aggregatedDiagnostics at job-completion merge time.
func (i *NodeDiagnostics) GetDroppedEventCount(ctx context.Context, jobID id.JobID) (*uint64, error) {
	if err := i.checkJobPermission(ctx, jobID); err != nil {
		return nil, err
	}

	if i.diagnosticsRepo == nil {
		return nil, nil
	}

	return i.diagnosticsRepo.FindJobSummary(ctx, jobID)
}
