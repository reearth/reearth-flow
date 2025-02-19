package gql

import (
	"context"
	"time"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

type LogLoader struct {
	usecase interfaces.Log
}

func NewLogLoader(usecase interfaces.Log) *LogLoader {
	return &LogLoader{usecase: usecase}
}

func (l *LogLoader) GetLogs(ctx context.Context, since time.Time, jobID gqlmodel.ID) ([]*gqlmodel.Log, error) {
	newJobID, err := id.JobIDFrom(string(jobID))
	if err != nil {
		return nil, err
	}
	res, err := l.usecase.GetLogs(ctx, since, newJobID, getOperator(ctx))
	if err != nil {
		return nil, err
	}

	logs := make([]*gqlmodel.Log, 0, len(res))
	for _, log := range res {
		logs = append(logs, gqlmodel.ToLog(log))
	}
	return logs, nil
}
