package gateway

import (
	"context"

	"github.com/reearth/reearth-flow/subscriber/pkg/stdoutlog"
)

type StdoutLogStorage interface {
	SaveToRedis(ctx context.Context, event *stdoutlog.Event) error
}
