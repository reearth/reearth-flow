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
	"github.com/reearth/reearth-flow/subscriber/pkg/diagnostic"
)

type DiagnosticSubscriber struct {
	sub     Subscription
	useCase interactor.DiagnosticSubscriberUseCase
	tracer  trace.Tracer
}

func NewDiagnosticSubscriber(subscription Subscription, useCase interactor.DiagnosticSubscriberUseCase) *DiagnosticSubscriber {
	return &DiagnosticSubscriber{
		sub:     subscription,
		useCase: useCase,
		tracer:  otel.Tracer(tracerName),
	}
}

func (s *DiagnosticSubscriber) StartListening(ctx context.Context) error {
	return s.sub.Receive(ctx, func(ctx context.Context, m Message) {
		defer func() {
			if r := recover(); r != nil {
				log.Printf("[DiagnosticSubscriber] panic recovered: %v", r)
			}
		}()

		ctx, span := s.tracer.Start(ctx, "DiagnosticSubscriber.ProcessMessage",
			trace.WithAttributes(
				attribute.String("subscriber.type", "diagnostic"),
			),
		)
		defer span.End()

		var evt diagnostic.DiagnosticEvent
		if err := json.Unmarshal(m.Data(), &evt); err != nil {
			log.Printf("[DiagnosticSubscriber] failed to unmarshal message: %v", err)
			span.SetStatus(codes.Error, "failed to unmarshal message")
			span.RecordError(err)
			m.Nack()
			return
		}

		nodeID := "<none>"
		if evt.NodeID != nil {
			nodeID = *evt.NodeID
		}
		log.Printf("[DiagnosticSubscriber] received event: workflow=%s, job=%s, code=%s, severity=%s, node=%s",
			evt.WorkflowID, evt.JobID, evt.Code, evt.Severity, nodeID)

		span.SetAttributes(
			attribute.String("diagnostic.workflow_id", evt.WorkflowID),
			attribute.String("diagnostic.job_id", evt.JobID),
			attribute.String("diagnostic.code", evt.Code),
			attribute.String("diagnostic.severity", evt.Severity),
			attribute.String("diagnostic.node_id", nodeID),
		)

		if err := s.useCase.ProcessDiagnosticEvent(ctx, &evt); err != nil {
			log.Printf("[DiagnosticSubscriber] failed to process diagnostic event: %v", err)
			span.SetStatus(codes.Error, "failed to process diagnostic event")
			span.RecordError(err)
			m.Nack()
			return
		}

		span.SetStatus(codes.Ok, "diagnostic event processed successfully")
		m.Ack()
	})
}
