package interactor

import (
	"context"
	"time"

	"github.com/reearth/reearth-flow/api/internal/usecase"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/log"
)

type LogInteractor struct {
	logsGatewayRedis gateway.Log
	logsGatewayGCS   gateway.Log
}

func NewLogInteractor(lgRedis gateway.Log, lgGCS gateway.Log) interfaces.Log {
	return &LogInteractor{
		logsGatewayRedis: lgRedis,
		logsGatewayGCS:   lgGCS,
	}
}

func (li *LogInteractor) GetLogs(ctx context.Context, since time.Time, workflowID id.WorkflowID, jobID id.JobID, operator *usecase.Operator) ([]*log.Log, error) {
	if time.Since(since) <= 60*time.Minute {
		return li.logsGatewayRedis.GetLogs(ctx, since, workflowID, jobID)
	} else {
		return li.logsGatewayGCS.GetLogs(ctx, since, workflowID, jobID)
	}
}
