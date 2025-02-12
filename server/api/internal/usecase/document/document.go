package document

import (
	"context"
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
		log.Infof("Creating new document client with WebSocket URL: %s", cfg.WebsocketServerURL)
		defaultClient = NewClient(cfg.WebsocketServerURL)
	})
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
	return getDefaultClient().GetLatest(ctx, id)
}

func GetHistory(ctx context.Context, id string) ([]*History, error) {
	return getDefaultClient().GetHistory(ctx, id)
}
