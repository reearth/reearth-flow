package pubsub

import (
	"context"
	"encoding/json"
	"log"

	"github.com/reearth/reearth-flow/subscriber/internal/usecase/interactor"
	"github.com/reearth/reearth-flow/subscriber/pkg/stdoutlog"
)

type StdoutLogSubscriber struct {
	subscription Subscription
	useCase      interactor.StdoutLogUseCase
}

func NewStdoutLogSubscriber(sub Subscription, uc interactor.StdoutLogUseCase) *StdoutLogSubscriber {
	return &StdoutLogSubscriber{
		subscription: sub,
		useCase:      uc,
	}
}

func (s *StdoutLogSubscriber) StartListening(ctx context.Context) error {
	log.Printf("INFO: StdoutLogSubscriber: Starting to listen on subscription")
	return s.subscription.Receive(ctx, func(ctx context.Context, msg Message) {
		log.Printf("DEBUG: StdoutLogSubscriber: Received message, DataLen=%d", len(msg.Data()))

		var event stdoutlog.Event
		if err := json.Unmarshal(msg.Data(), &event); err != nil {
			log.Printf("ERROR: StdoutLogSubscriber: Failed to unmarshal message data. Data: %s, Error: %v", string(msg.Data()), err)
			msg.Ack()
			return
		}

		log.Printf("DEBUG: StdoutLogSubscriber: Successfully unmarshalled event: %+v", event)

		if err := s.useCase.Process(ctx, &event); err != nil {
			log.Printf("ERROR: StdoutLogSubscriber: Failed to process event (Event JobID: %s): %v", event.JobID, err)
			msg.Nack()
			return
		}

		log.Printf("INFO: StdoutLogSubscriber: Successfully processed event (Event JobID: %s)", event.JobID)
		msg.Ack()
	})
}
