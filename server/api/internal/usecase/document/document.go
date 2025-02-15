package document

import (
	"context"
	"fmt"
	"sync"
	"time"

	"github.com/reearth/reearthx/log"
)

var (
	defaultClient *Client
	cfg           Config
	clientOnce    sync.Once
)

func Init(c Config) {
	cfg = c
}

func getDefaultClient() *Client {
	clientOnce.Do(func() {
		log.Infof("Creating new document client with gRPC address: %s", cfg.GrpcServerURL)
		client, err := NewClient(cfg.GrpcServerURL)
		if err != nil {
			log.Errorf("Failed to create document client: %v", err)
			return
		}
		defaultClient = client
	})
	if defaultClient == nil {
		log.Error("Document client is not initialized")
	}
	return defaultClient
}

type Document struct {
	ID        string
	Update    []int
	Clock     int
	Timestamp time.Time
}

type History struct {
	Update    []int
	Clock     int
	Timestamp time.Time
}

func GetLatest(ctx context.Context, id string) (*Document, error) {
	client := getDefaultClient()
	if client == nil {
		return nil, fmt.Errorf("document client is not initialized")
	}
	return client.GetLatest(ctx, id)
}

func GetHistory(ctx context.Context, id string) ([]*History, error) {
	client := getDefaultClient()
	if client == nil {
		return nil, fmt.Errorf("document client is not initialized")
	}
	return client.GetHistory(ctx, id)
}

func Rollback(ctx context.Context, id string, clock int) (*Document, error) {
	client := getDefaultClient()
	if client == nil {
		return nil, fmt.Errorf("document client is not initialized")
	}
	return client.Rollback(ctx, id, clock)
}
