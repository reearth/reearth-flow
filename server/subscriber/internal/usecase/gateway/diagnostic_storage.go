package gateway

import (
	"context"

	"github.com/reearth/reearth-flow/subscriber/pkg/diagnostic"
)

type DiagnosticStorage interface {
	Save(ctx context.Context, event *diagnostic.DiagnosticEvent) error
	SaveToMongo(ctx context.Context, event *diagnostic.DiagnosticEvent) error
}
