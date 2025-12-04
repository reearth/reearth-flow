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
	"github.com/reearth/reearth-flow/subscriber/pkg/userfacinglog"
)

type UserFacingLogSubscriber struct {
	sub     Subscription
	useCase interactor.UserFacingLogSubscriberUseCase
	tracer  trace.Tracer
}

func NewUserFacingLogSubscriber(subscription Subscription, useCase interactor.UserFacingLogSubscriberUseCase) *UserFacingLogSubscriber {
	return &UserFacingLogSubscriber{
		sub:     subscription,
		useCase: useCase,
		tracer:  otel.Tracer(tracerName),
	}
}

func (s *UserFacingLogSubscriber) StartListening(ctx context.Context) error {
	return s.sub.Receive(ctx, func(ctx context.Context, m Message) {
		defer func() {
			if r := recover(); r != nil {
				log.Printf("[UserFacingLogSubscriber] panic recovered: %v", r)
			}
		}()

		ctx, span := s.tracer.Start(ctx, "UserFacingLogSubscriber.ProcessMessage",
			trace.WithAttributes(
				attribute.String("subscriber.type", "user_facing_log"),
			),
		)
		defer span.End()

		var evt userfacinglog.UserFacingLogEvent
		if err := json.Unmarshal(m.Data(), &evt); err != nil {
			log.Printf("[UserFacingLogSubscriber] failed to unmarshal message: %v", err)
			span.SetStatus(codes.Error, "failed to unmarshal message")
			span.RecordError(err)
			m.Nack()
			return
		}

		// Log the received event for debugging
		nodeName := "<none>"
		if evt.NodeName != nil {
			nodeName = *evt.NodeName
		}
		log.Printf("[UserFacingLogSubscriber] received event: workflow=%s, job=%s, level=%s, node=%s",
			evt.WorkflowID, evt.JobID, evt.Level, nodeName)

		span.SetAttributes(
			attribute.String("user_facing_log.workflow_id", evt.WorkflowID),
			attribute.String("user_facing_log.job_id", evt.JobID),
			attribute.String("user_facing_log.level", string(evt.Level)),
			attribute.String("user_facing_log.node_name", nodeName),
		)

		if err := s.useCase.ProcessUserFacingLogEvent(ctx, &evt); err != nil {
			log.Printf("[UserFacingLogSubscriber] failed to process user facing log event: %v", err)
			span.SetStatus(codes.Error, "failed to process user facing log event")
			span.RecordError(err)
			m.Nack()
			return
		}

		span.SetStatus(codes.Ok, "user facing log event processed successfully")
		m.Ack()
	})
}
