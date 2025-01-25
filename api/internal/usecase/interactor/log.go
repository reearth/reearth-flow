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
	logsGatewayRedis    gateway.Log
	logsGatewayGCS      gateway.Log
	recentLogsThreshold time.Duration
}

func NewLogInteractor(lgRedis gateway.Log, lgGCS gateway.Log, recentLogsThreshold time.Duration) interfaces.Log {
	if recentLogsThreshold <= 0 {
		recentLogsThreshold = 60 * time.Minute
	}

	return &LogInteractor{
		logsGatewayRedis:    lgRedis,
		logsGatewayGCS:      lgGCS,
		recentLogsThreshold: recentLogsThreshold,
	}
}

func (li *LogInteractor) GetLogs(ctx context.Context, since time.Time, workflowID id.WorkflowID, jobID id.JobID, operator *usecase.Operator) ([]*log.Log, error) {
	// Add timeout to prevent long-running queries
	ctx, cancel := context.WithTimeout(ctx, 30*time.Second)
	defer cancel()
	until := time.Now().UTC()
	if time.Since(since) <= li.recentLogsThreshold {
		if li.logsGatewayRedis == nil {
			reearth_log.Error("logsGatewayRedis is nil: unable to get logs from Redis")
			return nil, fmt.Errorf("logsGatewayRedis is nil: unable to get logs from Redis")
		}
		logs, err := li.logsGatewayRedis.GetLogs(ctx, since, until, workflowID, jobID)
		if err != nil {
			return nil, fmt.Errorf("failed to get logs from Redis: %w", err)
		}
		return logs, nil
	}
	if li.logsGatewayGCS == nil {
		reearth_log.Error("logsGatewayGCS is nil: unable to get logs from GCS")
		return nil, fmt.Errorf("logsGatewayGCS is nil: unable to get logs from GCS")
	}
	logs, err := li.logsGatewayGCS.GetLogs(ctx, since, until, workflowID, jobID)
	if err != nil {
		return nil, fmt.Errorf("failed to get logs from GCS: %w", err)
	}
	return logs, nil
}
