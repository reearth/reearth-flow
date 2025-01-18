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
)

type LogInteractor struct {
	logsGatewayRedis    gateway.Log
	logsGatewayGCS      gateway.Log
	recentLogsThreshold time.Duration
}

func NewLogInteractor(lgRedis gateway.Log, lgGCS gateway.Log, recentLogsThreshold time.Duration) (interfaces.Log, error) {
	if lgRedis == nil || lgGCS == nil {
		return nil, fmt.Errorf("log gateways are required")
	}
	if recentLogsThreshold <= 0 {
		recentLogsThreshold = 60 * time.Minute
	}

	return &LogInteractor{
		logsGatewayRedis:    lgRedis,
		logsGatewayGCS:      lgGCS,
		recentLogsThreshold: recentLogsThreshold,
	}, nil
}

func (li *LogInteractor) GetLogs(ctx context.Context, since time.Time, workflowID id.WorkflowID, jobID id.JobID, operator *usecase.Operator) ([]*log.Log, error) {
	// Add timeout to prevent long-running queries
	ctx, cancel := context.WithTimeout(ctx, 30*time.Second)
	defer cancel()
	if time.Since(since) <= li.recentLogsThreshold {
		logs, err := li.logsGatewayRedis.GetLogs(ctx, since, workflowID, jobID)
		if err != nil {
			return nil, fmt.Errorf("failed to get logs from Redis: %w", err)
		}
		return logs, nil
	}
	logs, err := li.logsGatewayGCS.GetLogs(ctx, since, workflowID, jobID)
	if err != nil {
		return nil, fmt.Errorf("failed to get logs from GCS: %w", err)
	}
	return logs, nil
}
