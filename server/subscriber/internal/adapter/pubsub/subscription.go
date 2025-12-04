package pubsub

import (
	"context"

	"cloud.google.com/go/pubsub/v2"
	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/codes"
	"go.opentelemetry.io/otel/propagation"
	semconv "go.opentelemetry.io/otel/semconv/v1.26.0"
	"go.opentelemetry.io/otel/trace"
)

const tracerName = "github.com/reearth/reearth-flow/subscriber/internal/adapter/pubsub"

type Subscription interface {
	Receive(ctx context.Context, f func(context.Context, Message)) error
}

type realSubscription struct {
	sub            *pubsub.Subscriber
	subscriptionID string
	tracer         trace.Tracer
}

func NewRealSubscription(sub *pubsub.Subscriber) Subscription {
	return &realSubscription{
		sub:            sub,
		subscriptionID: sub.String(),
		tracer:         otel.Tracer(tracerName),
	}
}

func (r *realSubscription) Receive(ctx context.Context, f func(context.Context, Message)) error {
	return r.sub.Receive(ctx, func(ctx context.Context, m *pubsub.Message) {
		ctx = r.extractTraceContext(ctx, m)

		ctx, span := r.tracer.Start(ctx, "pubsub.receive",
			trace.WithSpanKind(trace.SpanKindConsumer),
			trace.WithAttributes(
				semconv.MessagingSystemGCPPubsub,
				semconv.MessagingDestinationName(r.subscriptionID),
				semconv.MessagingMessageID(m.ID),
				attribute.Int("messaging.message.body.size", len(m.Data)),
			),
		)

		msg := NewRealMessage(m)

		defer func() {
			if r := recover(); r != nil {
				span.SetStatus(codes.Error, "panic recovered")
				span.RecordError(nil)
				span.End()
				panic(r)
			}
			span.End()
		}()

		f(ctx, msg)
	})
}

func (r *realSubscription) extractTraceContext(ctx context.Context, m *pubsub.Message) context.Context {
	if m.Attributes == nil {
		return ctx
	}

	propagator := otel.GetTextMapPropagator()
	carrier := propagation.MapCarrier(m.Attributes)
	return propagator.Extract(ctx, carrier)
}
