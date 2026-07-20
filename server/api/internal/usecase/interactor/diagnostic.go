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
// Backend policy: Redis first, Mongo fallback. The subscriber writes every
// diagnostic to both stores (see server/subscriber's diagnostic ingestion),
// but Redis lists carry a 24h TTL while the Mongo nodeDiagnostics collection
// is durable. So: try Redis (fresh/live — this is where a still-running or
// recently-finished job's diagnostics live); if Redis returned nothing
// (empty, TTL-expired, or erroring), fall back to Mongo (works for
// long-finished/terminal jobs, and for job-completion merge-persisted
// failed-node/aggregated rows, which are ONLY ever written to Mongo — see
// interactor/job.go's persistTerminalDiagnostics). This mirrors
// interactor/node.go's Redis-live / Mongo-durable split, but as a single
// fallback chain rather than two separate methods: callers here always want
// "whatever we still have," not NodeExecution's live-vs-terminal
// distinction.
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

	if i.redisGateway != nil {
		rows, err := i.redisGateway.GetNodeDiagnostics(ctx, jobID, nodeID)
		if err != nil {
			log.Warnfc(ctx, "diagnostic: failed to get node diagnostics from Redis: %v", err)
		} else if len(rows) > 0 {
			return rows, nil
		}
	}

	if i.diagnosticsRepo == nil {
		return nil, nil
	}

	rows, err := i.diagnosticsRepo.FindByJobNodeID(ctx, jobID, nodeID)
	if err != nil {
		return nil, err
	}
	return rows, nil
}

func (i *NodeDiagnostics) GetJobDiagnostics(ctx context.Context, jobID id.JobID) ([]*diagnostic.Diagnostic, error) {
	if err := i.checkJobPermission(ctx, jobID); err != nil {
		return nil, err
	}

	if i.redisGateway != nil {
		rows, err := i.redisGateway.GetJobDiagnostics(ctx, jobID)
		if err != nil {
			log.Warnfc(ctx, "diagnostic: failed to get job diagnostics from Redis: %v", err)
		} else if len(rows) > 0 {
			return rows, nil
		}
	}

	if i.diagnosticsRepo == nil {
		return nil, nil
	}

	rows, err := i.diagnosticsRepo.FindByJobID(ctx, jobID)
	if err != nil {
		return nil, err
	}
	return rows, nil
}

// fatalEffectiveDisposition is the wire/domain string value of the engine's
// Disposition::Fatal (Rust #[serde(rename_all = "snake_case")]). The engine
// guarantees every RunSummary::failed_nodes entry is stamped
// effective_disposition = Some(Fatal) (dag_executor.rs's fold_outcomes) and
// that RunSummary::aggregated_diagnostics entries are NEVER Fatal
// (job_complete_event.json's aggregatedDiagnostics field contract) — so this
// is a precise, lossless way to recover "which wire array a persisted row
// came from" after both have landed as indistinguishable DiagnosticDocument
// rows in the same nodeDiagnostics collection (see GetFailedNodes).
const fatalEffectiveDisposition = "fatal"

// GetFailedNodes reads the job's terminal per-node fatal-failure rows
// (GraphQL Job.failedNodes). Deliberately Mongo-only (diagnosticsRepo),
// never Redis: failedNodes/aggregatedDiagnostics rows are persisted
// exclusively at job-completion merge time (interactor/job.go's
// persistTerminalDiagnostics) and are never written to Redis, so consulting
// Redis here (as GetJobDiagnostics does for live per-event diagnostics)
// would be reading the wrong store. FindByJobID returns both failedNodes-
// and aggregatedDiagnostics-derived rows undifferentiated (both carry a
// "code" field); the fatalEffectiveDisposition filter recovers exactly the
// failedNodes subset.
func (i *NodeDiagnostics) GetFailedNodes(ctx context.Context, jobID id.JobID) ([]*diagnostic.Diagnostic, error) {
	if err := i.checkJobPermission(ctx, jobID); err != nil {
		return nil, err
	}

	if i.diagnosticsRepo == nil {
		return nil, nil
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
	return failed, nil
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
