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
// GetFailedNodes/GetDroppedEventCount are thin non-batching wrappers, like
// LogLoader/NodeExLoader. GetNodeDiagnostics is not: it batches via
// jobDiagnosticsFetch below to avoid an N+1 across a job's node list.
type DiagnosticLoader struct {
	usecase  interfaces.NodeDiagnostics
	jobFetch map[id.JobID]*jobDiagnosticsFetch
	mu       sync.Mutex
}

// jobDiagnosticsFetch memoizes one in-flight/completed GetJobDiagnostics
// call per jobID, so concurrent GetNodeDiagnostics calls for the same job
// share a single fetch instead of issuing one each.
type jobDiagnosticsFetch struct {
	err  error
	done chan struct{}
	rows []*diagnostic.Diagnostic
}

func NewDiagnosticLoader(usecase interfaces.NodeDiagnostics) *DiagnosticLoader {
	return &DiagnosticLoader{usecase: usecase}
}

// GetNodeDiagnostics backs NodeExecution.diagnostics. Avoids an N+1 by
// fetching the whole job's diagnostics once (via loadJobDiagnostics) and
// partitioning by nodeID in memory — safe because GetJobDiagnostics is a
// strict superset of any single node's rows, with the same permission scope.
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

// loadJobDiagnostics fetches and memoizes GetJobDiagnostics(jobID) once per
// (loader instance, jobID) pair; concurrent callers for the same jobID
// block on the in-flight fetch.
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
