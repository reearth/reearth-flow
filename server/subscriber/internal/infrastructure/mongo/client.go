package mongo

import (
	"context"
	"time"

	"go.mongodb.org/mongo-driver/mongo"
	"go.mongodb.org/mongo-driver/mongo/options"
	"go.opentelemetry.io/contrib/instrumentation/go.mongodb.org/mongo-driver/mongo/otelmongo"
)

type MongoClient interface {
	Collection(name string) *mongo.Collection
	Disconnect(ctx context.Context) error
	Ping(ctx context.Context) error
}

type realMongoClient struct {
	client       *mongo.Client
	databaseName string
}

func NewMongoClient(ctx context.Context, uri string, databaseName string) (MongoClient, error) {
	client, err := mongo.Connect(
		ctx,
		options.Client().
			ApplyURI(uri).
			SetMonitor(otelmongo.NewMonitor()).
			SetConnectTimeout(10 * time.Second).
			SetServerSelectionTimeout(5 * time.Second),
	)
	if err != nil {
		return nil, err
	}

	if err := client.Ping(ctx, nil); err != nil {
		return nil, err
	}

	return &realMongoClient{
		client:       client,
		databaseName: databaseName,
	}, nil
}

func (m *realMongoClient) Disconnect(ctx context.Context) error {
	ctx, cancel := context.WithTimeout(ctx, 10*time.Second)
	defer cancel()
	return m.client.Disconnect(ctx)
}

func (m *realMongoClient) Ping(ctx context.Context) error {
	ctx, cancel := context.WithTimeout(ctx, 5*time.Second)
	defer cancel()
	return m.client.Database("admin").RunCommand(ctx, map[string]interface{}{"ping": 1}).Err()
}

func (m *realMongoClient) Collection(name string) *mongo.Collection {
	return m.client.Database(m.databaseName).Collection(name)
}
