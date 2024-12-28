package gateway

import (
	"context"

	domainLog "github.com/reearth/reearth-flow/log-subscriber/pkg/log"
)

type LogStorage interface {
	SaveToRedis(ctx context.Context, event *domainLog.LogEvent) error
	SaveToGCS(ctx context.Context, event *domainLog.LogEvent) error
}