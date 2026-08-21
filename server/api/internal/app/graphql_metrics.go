package app

import (
	"context"
	"sync/atomic"
	"time"

	"github.com/99designs/gqlgen/graphql"
	apiotel "github.com/reearth/reearth-flow/api/internal/app/otel"
)

// metricsExtension records InstrumentRequestDuration/AccountsCalls/RedisCommands
// once per GraphQL operation, labelled by operation name.
type metricsExtension struct {
	instruments *apiotel.Instruments
}

var (
	_ graphql.HandlerExtension     = (*metricsExtension)(nil)
	_ graphql.OperationInterceptor = (*metricsExtension)(nil)
)

func (metricsExtension) ExtensionName() string { return "Metrics" }

func (metricsExtension) Validate(graphql.ExecutableSchema) error { return nil }

func (m *metricsExtension) InterceptOperation(ctx context.Context, next graphql.OperationHandler) graphql.ResponseHandler {
	if m.instruments == nil {
		return next(ctx)
	}

	ctx = apiotel.WithRequestCounters(ctx)
	start := time.Now()
	handler := next(ctx)

	var recorded atomic.Bool
	return func(ctx context.Context) *graphql.Response {
		resp := handler(ctx)
		if recorded.CompareAndSwap(false, true) {
			operation := "unknown"
			if opCtx := graphql.GetOperationContext(ctx); opCtx != nil && opCtx.OperationName != "" {
				operation = opCtx.OperationName
			}
			m.instruments.RecordRequest(ctx, operation, time.Since(start))
		}
		return resp
	}
}
