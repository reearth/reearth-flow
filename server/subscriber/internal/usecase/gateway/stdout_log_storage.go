package gateway

import (
	"context"

	"github.com/reearth/reearth-flow/subscriber/pkg/stdoutlog"
)

type StdoutLogStorage interface {
	Save(ctx context.Context, event *stdoutlog.Event) error
}
