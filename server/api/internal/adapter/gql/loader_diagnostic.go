package gql

import (
	"context"
	"sync"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/diagnostic"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

// DiagnosticLoader wraps interfaces.NodeDiagnostics for the GraphQL layer.
// GetFailedNodes/GetDroppedEventCount mirror the thin non-batching wrapper
// pattern used by LogLoader/NodeExLoader (each backs exactly one resolver
// field on a single parent object, called at most once per request).
// GetNodeDiagnostics does not: NodeExecution.diagnostics is resolved once
// per sibling NodeExecution in a job's node list, all sharing the same
// jobID, which was an N+1 (one permission check + one Redis/Mongo round
// trip per node) — see jobDiagnosticsFetch below.
type DiagnosticLoader struct {
	usecase  interfaces.NodeDiagnostics
	jobFetch map[id.JobID]*jobDiagnosticsFetch
	mu       sync.Mutex
}

// jobDiagnosticsFetch memoizes one in-flight or completed
// usecase.GetJobDiagnostics call for a jobID, so concurrent/sequential
// GetNodeDiagnostics calls for different nodes of the same job share a
// single fetch instead of issuing one each.
type jobDiagnosticsFetch struct {
	err  error
	done chan struct{}
	rows []*diagnostic.Diagnostic
}

func NewDiagnosticLoader(usecase interfaces.NodeDiagnostics) *DiagnosticLoader {
	return &DiagnosticLoader{usecase: usecase}
}

// GetNodeDiagnostics backs NodeExecution.diagnostics. Rather than calling
// usecase.GetNodeDiagnostics(jobID, nodeID) once per node (an N+1 across a
// job's node list — see resolver_nodeExecution.go), it fetches the whole
// job's diagnostics ONCE per DiagnosticLoader instance (a fresh instance is
// built per GraphQL request by NewLoaders/AttachUsecases — see context.go —
// so this cannot leak state across requests) via the already-merged/deduped
// usecase.GetJobDiagnostics, then partitions by nodeID in memory. This is
// correct because GetJobDiagnostics is a strict superset of any single
// node's rows: the subscriber pushes every diagnostic (node-scoped or
// job-level) onto both the per-node AND the whole-job Redis list (see
// server/subscriber's SaveDiagnosticToRedis), and Mongo's FindByJobID is
// unfiltered by node. Same permission scope too: both methods call the
// identical i.checkJobPermission(ctx, jobID).
func (l *DiagnosticLoader) GetNodeDiagnostics(ctx context.Context, jobID gqlmodel.ID, nodeID string) ([]*gqlmodel.Diagnostic, error) {
	jId, err := id.JobIDFrom(string(jobID))
	if err != nil {
		return nil, err
	}

	rows, err := l.loadJobDiagnostics(ctx, jId)
	if err != nil {
		return nil, err
	}

	filtered := make([]*diagnostic.Diagnostic, 0, len(rows))
	for _, row := range rows {
		rowNodeID := ""
		if row.NodeID() != nil {
			rowNodeID = *row.NodeID()
		}
		if rowNodeID == nodeID {
			filtered = append(filtered, row)
		}
	}
	return gqlmodel.ToDiagnostics(filtered), nil
}

// loadJobDiagnostics fetches and memoizes usecase.GetJobDiagnostics(jobID)
// once per (loader instance, jobID) pair. Concurrent callers for the same
// jobID block on the same in-flight fetch rather than issuing their own.
func (l *DiagnosticLoader) loadJobDiagnostics(ctx context.Context, jobID id.JobID) ([]*diagnostic.Diagnostic, error) {
	l.mu.Lock()
	fetch, ok := l.jobFetch[jobID]
	if ok {
		l.mu.Unlock()
		<-fetch.done
		return fetch.rows, fetch.err
	}

	fetch = &jobDiagnosticsFetch{done: make(chan struct{})}
	if l.jobFetch == nil {
		l.jobFetch = make(map[id.JobID]*jobDiagnosticsFetch)
	}
	l.jobFetch[jobID] = fetch
	l.mu.Unlock()

	fetch.rows, fetch.err = l.usecase.GetJobDiagnostics(ctx, jobID)
	close(fetch.done)

	return fetch.rows, fetch.err
}

// GetFailedNodes backs Job.failedNodes.
func (l *DiagnosticLoader) GetFailedNodes(ctx context.Context, jobID gqlmodel.ID) ([]*gqlmodel.Diagnostic, error) {
	jId, err := id.JobIDFrom(string(jobID))
	if err != nil {
		return nil, err
	}

	rows, err := l.usecase.GetFailedNodes(ctx, jId)
	if err != nil {
		return nil, err
	}
	return gqlmodel.ToDiagnostics(rows), nil
}

// GetDroppedEventCount backs Job.droppedEventCount.
func (l *DiagnosticLoader) GetDroppedEventCount(ctx context.Context, jobID gqlmodel.ID) (*int, error) {
	jId, err := id.JobIDFrom(string(jobID))
	if err != nil {
		return nil, err
	}

	count, err := l.usecase.GetDroppedEventCount(ctx, jId)
	if err != nil {
		return nil, err
	}
	if count == nil {
		return nil, nil
	}
	c := int(*count)
	return &c, nil
}
