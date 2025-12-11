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
	"github.com/reearth/reearth-flow/subscriber/pkg/node"
)

type NodeSubscriber struct {
	sub     Subscription
	useCase interactor.NodeSubscriberUseCase
	tracer  trace.Tracer
}

func NewNodeSubscriber(subscription Subscription, useCase interactor.NodeSubscriberUseCase) *NodeSubscriber {
	return &NodeSubscriber{
		sub:     subscription,
		useCase: useCase,
		tracer:  otel.Tracer(tracerName),
	}
}

func (s *NodeSubscriber) StartListening(ctx context.Context) error {
	log.Println("[NodeSubscriber] Starting to listen for node pass through events")

	return s.sub.Receive(ctx, func(ctx context.Context, m Message) {
		defer func() {
			if r := recover(); r != nil {
				log.Printf("[NodeSubscriber] panic recovered: %v", r)
			}
		}()

		ctx, span := s.tracer.Start(ctx, "NodeSubscriber.ProcessMessage",
			trace.WithAttributes(
				attribute.String("subscriber.type", "node"),
			),
		)
		defer span.End()

		var evt node.NodeStatusEvent
		if err := json.Unmarshal(m.Data(), &evt); err != nil {
			log.Printf("[NodeSubscriber] failed to unmarshal node event: %v", err)
			span.SetStatus(codes.Error, "failed to unmarshal node event")
			span.RecordError(err)
			m.Nack()
			return
		}

		span.SetAttributes(
			attribute.String("node.job_id", evt.JobID),
			attribute.String("node.node_id", evt.NodeID),
			attribute.String("node.status", string(evt.Status)),
		)

		if err := s.useCase.ProcessNodeEvent(ctx, &evt); err != nil {
			log.Printf("[NodeSubscriber] failed to process node event: %v", err)
			span.SetStatus(codes.Error, "failed to process node event")
			span.RecordError(err)
			m.Nack()
			return
		}

		span.SetStatus(codes.Ok, "node event processed successfully")
		m.Ack()
		log.Printf("[NodeSubscriber] Successfully processed event for job %s", evt.JobID)
	})
}
