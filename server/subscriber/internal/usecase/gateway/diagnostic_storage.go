package gateway

import (
	"context"

	"github.com/reearth/reearth-flow/subscriber/pkg/diagnostic"
)

type DiagnosticStorage interface {
	Save(ctx context.Context, event *diagnostic.DiagnosticEvent) error
	SaveDurable(ctx context.Context, event *diagnostic.DiagnosticEvent) error
}
