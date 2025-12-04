package pubsub

import (
	"context"
	"encoding/json"
	"log"

	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/codes"
	"go.opentelemetry.io/otel/trace"

	"github.com/reearth/reearth-flow/subscriber/internal/usecase/interactor"
	domainLog "github.com/reearth/reearth-flow/subscriber/pkg/log"
)

type LogSubscriber struct {
	sub     Subscription
	useCase interactor.LogSubscriberUseCase
	tracer  trace.Tracer
}

func NewLogSubscriber(subscription Subscription, useCase interactor.LogSubscriberUseCase) *LogSubscriber {
	return &LogSubscriber{
		sub:     subscription,
		useCase: useCase,
		tracer:  otel.Tracer(tracerName),
	}
}

func (s *LogSubscriber) StartListening(ctx context.Context) error {
	return s.sub.Receive(ctx, func(ctx context.Context, m Message) {
		defer func() {
			if r := recover(); r != nil {
				log.Printf("[LogSubscriber] panic recovered: %v", r)
			}
		}()

		ctx, span := s.tracer.Start(ctx, "LogSubscriber.ProcessMessage",
			trace.WithAttributes(
				attribute.String("subscriber.type", "log"),
			),
		)
		defer span.End()

		var evt domainLog.LogEvent
		if err := json.Unmarshal(m.Data(), &evt); err != nil {
			log.Printf("failed to unmarshal message: %v", err)
			span.SetStatus(codes.Error, "failed to unmarshal message")
			span.RecordError(err)
			m.Nack()
			return
		}

		span.SetAttributes(
			attribute.String("log.workflow_id", evt.WorkflowID),
			attribute.String("log.job_id", evt.JobID),
			attribute.String("log.level", string(evt.LogLevel)),
		)

		if err := s.useCase.ProcessLogEvent(ctx, &evt); err != nil {
			log.Printf("failed to process event: %v", err)
			span.SetStatus(codes.Error, "failed to process event")
			span.RecordError(err)
			m.Nack()
			return
		}

		span.SetStatus(codes.Ok, "message processed successfully")
		m.Ack()
	})
}
