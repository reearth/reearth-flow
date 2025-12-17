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
	domainJob "github.com/reearth/reearth-flow/subscriber/pkg/job"
)

type JobSubscriber struct {
	sub     Subscription
	useCase interactor.JobSubscriberUseCase
	tracer  trace.Tracer
}

func NewJobSubscriber(subscription Subscription, useCase interactor.JobSubscriberUseCase) *JobSubscriber {
	return &JobSubscriber{
		sub:     subscription,
		useCase: useCase,
		tracer:  otel.Tracer(tracerName),
	}
}

func (s *JobSubscriber) StartListening(ctx context.Context) error {
	return s.sub.Receive(ctx, func(ctx context.Context, m Message) {
		defer func() {
			if r := recover(); r != nil {
				log.Printf("[JobSubscriber] panic recovered: %v", r)
			}
		}()

		ctx, span := s.tracer.Start(ctx, "JobSubscriber.ProcessMessage",
			trace.WithAttributes(
				attribute.String("subscriber.type", "job"),
			),
		)
		defer span.End()

		var evt domainJob.JobCompleteEvent
		if err := json.Unmarshal(m.Data(), &evt); err != nil {
			log.Printf("failed to unmarshal message: %v", err)
			span.SetStatus(codes.Error, "failed to unmarshal message")
			span.RecordError(err)
			m.Nack()
			return
		}

		span.SetAttributes(
			attribute.String("job.job_id", evt.JobID),
			attribute.String("job.workflow_id", evt.WorkflowID),
			attribute.String("job.result", evt.Result),
		)

		if err := s.useCase.ProcessJobCompleteEvent(ctx, &evt); err != nil {
			log.Printf("failed to process event: %v", err)
			span.SetStatus(codes.Error, "failed to process event")
			span.RecordError(err)
			m.Nack()
			return
		}

		span.SetStatus(codes.Ok, "job event processed successfully")
		m.Ack()
	})
}
