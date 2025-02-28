package websocket

import (
	"context"
	"fmt"
	"sync"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/websocket"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	ws "github.com/reearth/reearth-flow/api/pkg/websocket"
	"github.com/reearth/reearthx/log"
)

var (
	defaultClient interfaces.WebsocketClient
	clientConfig  websocket.Config
	clientOnce    sync.Once
)

func Init(gcsBucket string, gcsEndpoint *string) {
	clientConfig = websocket.Config{
		GcsBucket:   gcsBucket,
		GcsEndpoint: gcsEndpoint,
	}
}

func getDefaultClient() interfaces.WebsocketClient {
	clientOnce.Do(func() {
		log.Info("Creating new document client")
		client, err := websocket.NewClient(clientConfig)
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

func GetLatest(ctx context.Context, id string) (*ws.Document, error) {
	client := getDefaultClient()
	if client == nil {
		return nil, fmt.Errorf("document client is not initialized")
	}
	return client.GetLatest(ctx, id)
}

func GetHistory(ctx context.Context, id string) ([]*ws.History, error) {
	client := getDefaultClient()
	if client == nil {
		return nil, fmt.Errorf("document client is not initialized")
	}
	return client.GetHistory(ctx, id)
}

func Rollback(ctx context.Context, id string, version int) (*ws.Document, error) {
	client := getDefaultClient()
	if client == nil {
		return nil, fmt.Errorf("document client is not initialized")
	}
	return client.Rollback(ctx, id, version)
}
