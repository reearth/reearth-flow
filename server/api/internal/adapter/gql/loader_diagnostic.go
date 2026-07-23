package gql

import (
	"context"
	"sync"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/diagnostic"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

type DiagnosticLoader struct {
	usecase  interfaces.NodeDiagnostics
	jobFetch map[id.JobID]*jobDiagnosticsFetch
	mu       sync.Mutex
}

type jobDiagnosticsFetch struct {
	err  error
	done chan struct{}
	rows []*diagnostic.Diagnostic
}

func NewDiagnosticLoader(usecase interfaces.NodeDiagnostics) *DiagnosticLoader {
	return &DiagnosticLoader{usecase: usecase}
}

// Safe because GetJobDiagnostics is a strict superset of any single node's
// rows, with the same permission scope.
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
