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
// Backend policy: Redis (live, 24h TTL) MERGED with Mongo (durable —
// mirrors every live diagnostic.v1 row the subscriber ingests, plus the
// job-completion job-complete.v1 terminal rows persisted by
// interactor/job.go's persistTerminalDiagnostics, which are ONLY ever
// written to Mongo, never Redis). Both sources are always consulted — this
// used to short-circuit on a non-empty Redis result, which hid Mongo-only
// terminal rows (a node's terminal fatal row) until the Redis TTL expired.
// The merge is deduped by dedupeDiagnostics: a whole-2b review of the engine
// confirmed Fatal diagnostics are NEVER published as live events (neither
// report()'s nor report_drop()'s Fatal branch calls event_hub.diagnostic() —
// they only call record_fatal, see executor_operation.rs/diagnostics.rs; the
// per-feature process()-error fatal Task 6 added mirrors this), so a
// failedNodes-derived (":failed:") row has no live counterpart to merge away
// — it exists ONLY as a terminal row. The genuine duplicate dedupeDiagnostics
// fixes is the AGGREGATED SUMMARIES: emit_summaries (diagnostics.rs)
// publishes each finish()-time WarnDrop/Reject/WarnContinue summary live
// (ingested as a diagnostic.v1 row) AND returns the same summaries for
// folding into RunSummary.aggregated_diagnostics, which persistTerminalDiagnostics
// persists again as an aggregatedDiagnostics-derived (":aggregated:") terminal
// row at job completion — so the same aggregated diagnostic can legitimately
// appear as both a live diagnostic.v1 row and a terminal job-complete.v1 row;
// the dedupe collapses that pair to the terminal copy without dropping
// anything else (distinct live warnings/rejects have distinct keys and pass
// through unchanged). dedupeDiagnostics' key includes effectiveDisposition
// precisely so a failedNodes row and an aggregatedDiagnostics row that share
// a (nodeId, code) — a real possibility, e.g. one call site warn()s a code
// while another report()s the same code to Fatal — are never mistaken for
// that live/terminal pair and collapsed into each other.
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
// would be reading the wrong store. FindByJobID returns every schema-tagged
// row undifferentiated (failedNodes-derived, aggregatedDiagnostics-derived,
// AND the subscriber's own live-mirrored diagnostic.v1 rows all carry a
// "code" field); the fatalEffectiveDisposition filter recovers the fatal
// subset. Fatal diagnostics are never published live (see the package doc
// comment above), so every row that survives the filter is already a
// failedNodes-derived (":failed:") terminal row with no live counterpart to
// merge — dedupeDiagnostics here is a defensive backstop (e.g. against a
// stray row sharing a node/code/disposition with a genuine one) rather than
// the live/terminal merge GetNodeDiagnostics/GetJobDiagnostics rely on.
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

// dedupeDiagnostics collapses rows that share the same (nodeId, code,
// effectiveDisposition) key down to one representative row, keeping input
// order for the first occurrence of each key. It exists because Mongo (and,
// for GetNodeDiagnostics/GetJobDiagnostics, the Redis+Mongo merge) can
// legitimately hold two rows for the very same reported diagnostic: a live
// row mirroring the subscriber's per-event diagnostic.v1 ingestion, and a
// terminal job-complete.v1 row persisted at job-completion merge time (see
// interactor/job.go's persistTerminalDiagnostics). That pairing is
// aggregated-summary-only, not fatal-only (see the package doc comment
// above): emit_summaries (diagnostics.rs) publishes every finish()-time
// WarnDrop/Reject/WarnContinue summary live AND returns it for folding into
// RunSummary.aggregated_diagnostics, so the same summary legitimately lands
// in Mongo twice — once as its live diagnostic.v1 mirror, once as its
// aggregatedDiagnostics-derived (":aggregated:") terminal row. Fatal
// diagnostics never take this path (report()/report_drop()'s Fatal branches,
// and Task 6's per-feature process()-error fatal, only ever call
// record_fatal — never event_hub.diagnostic()), so a failedNodes-derived
// (":failed:") row has no live counterpart to collapse with.
//
// effectiveDisposition is part of the key, not just (nodeId, code), because
// a ":failed:" row and an ":aggregated:" row CAN legitimately share a
// (nodeId, code) — e.g. one call site warn()s a code (always aggregates,
// skips resolve()) while another report()s the very same code and the
// policy resolves it to Fatal there. Without effectiveDisposition in the
// key, preferOver's Terminal+Timestamp tie-break would collapse those two
// unrelated rows into one, nondeterministically dropping whichever loses —
// a real fatal-failure row or a real aggregated-drop count, either of which
// is a silent loss of distinct information, not a duplicate.
//
// preferOver decides the tie-break: a Terminal() row always wins over a
// non-terminal one for the same key (it is the durable, authoritative copy
// once a job has completed); among two rows of equal terminality, the more
// recent Timestamp wins (live diagnostics can be periodic/aggregated
// snapshots under the same code — see report()'s WarnDrop/Reject
// aggregation via DiagnosticsHandle.record — so the latest snapshot carries
// the fullest count). Rows with no counterpart for their key pass through
// unchanged.
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
// dedupeDiagnostics' representative row for a (nodeId, code,
// effectiveDisposition) key.
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
