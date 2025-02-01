package interactor

import (
	"context"
	"fmt"
	"time"

	"github.com/reearth/reearth-flow/api/internal/usecase"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/log"
	reearth_log "github.com/reearth/reearthx/log"
)

type LogInteractor struct {
	logsGatewayRedis gateway.Log
}

func NewLogInteractor(lgRedis gateway.Log) interfaces.Log {
	return &LogInteractor{
		logsGatewayRedis: lgRedis,
	}
}

func (li *LogInteractor) GetLogs(ctx context.Context, since time.Time, jobID id.JobID, operator *usecase.Operator) ([]*log.Log, error) {
	// Add timeout to prevent long-running queries
	ctx, cancel := context.WithTimeout(ctx, 30*time.Second)
	defer cancel()
	until := time.Now().UTC()
	if li.logsGatewayRedis == nil {
		reearth_log.Error("logsGatewayRedis is nil: unable to get logs from Redis")
		return nil, fmt.Errorf("logsGatewayRedis is nil: unable to get logs from Redis")
	}
	logs, err := li.logsGatewayRedis.GetLogs(ctx, since, until, jobID)
	if err != nil {
		return nil, fmt.Errorf("failed to get logs from Redis: %w", err)
	}
	return logs, nil

}
