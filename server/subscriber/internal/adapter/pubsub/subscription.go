package pubsub

import (
	"context"

	"cloud.google.com/go/pubsub"
)

type Subscription interface {
	Receive(ctx context.Context, f func(context.Context, Message)) error
}

type realSubscription struct {
	sub *pubsub.Subscription
}

func NewRealSubscription(sub *pubsub.Subscription) Subscription {
	return &realSubscription{sub: sub}
}

func (r *realSubscription) Receive(ctx context.Context, f func(context.Context, Message)) error {
	return r.sub.Receive(ctx, func(ctx context.Context, m *pubsub.Message) {
		f(ctx, NewRealMessage(m))
	})
}
