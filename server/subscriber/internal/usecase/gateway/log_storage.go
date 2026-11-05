package gateway

import (
	"context"

	domainLog "github.com/reearth/reearth-flow/subscriber/pkg/log"
)

type LogStorage interface {
	Save(ctx context.Context, event *domainLog.LogEvent) error
}
