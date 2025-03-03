package websocket

import (
	"context"
	"fmt"
	"time"

	"github.com/apache/thrift/lib/go/thrift"
	"github.com/reearth/reearth-flow/api/pkg/websocket"
	"github.com/reearth/reearth-flow/api/proto"
	"github.com/reearth/reearthx/log"
)

type Config struct {
	ServerURL string `json:"server_url"`
}

type Client struct {
	config         Config
	documentClient *proto.DocumentServiceClient
	transport      thrift.TTransport
}

func NewClient(config Config) (*Client, error) {
	if config.ServerURL == "" {
		config.ServerURL = "http://localhost:8000"
	}

	docEndpoint := config.ServerURL + "/doc"
	trans, err := thrift.NewTHttpClient(docEndpoint)
	if err != nil {
		return nil, fmt.Errorf("failed to create Thrift HTTP client: %w", err)
	}

	cfg := &thrift.TConfiguration{}
	framedTransport := thrift.NewTFramedTransportConf(trans, cfg)

	protocolFactory := thrift.NewTBinaryProtocolFactoryConf(cfg)

	documentClient := proto.NewDocumentServiceClientFactory(framedTransport, protocolFactory)

	return &Client{config: config,
		documentClient: documentClient,
		transport:      framedTransport,
	}, nil
}

func (c *Client) Close() error {
	return c.transport.Close()
}

func (c *Client) GetLatest(ctx context.Context, docID string) (*websocket.Document, error) {
	if !c.transport.IsOpen() {
		if err := c.transport.Open(); err != nil {
			return nil, fmt.Errorf("failed to open transport: %w", err)
		}
	}

	request := &proto.GetLatestRequest{
		DocID: docID,
	}

	response, err := c.documentClient.GetLatest(ctx, request)
	if err != nil {
		return nil, fmt.Errorf("failed to get latest document: %w", err)
	}

	if response == nil || response.Document == nil {
		return nil, fmt.Errorf("received empty response")
	}

	updates := make([]int, len(response.Document.Updates))
	for i, update := range response.Document.Updates {
		updates[i] = int(update)
	}

	timestamp, err := time.Parse(time.RFC3339, response.Document.Timestamp)
	if err != nil {
		log.Warnf("failed to parse timestamp: %v, using current time", err)
		timestamp = time.Now()
	}

	doc := &websocket.Document{
		ID:        response.Document.ID,
		Updates:   updates,
		Version:   int(response.Document.Version),
		Timestamp: timestamp,
	}

	log.Infof("Returning document: %+v", doc)
	return doc, nil
}

func (c *Client) GetHistory(ctx context.Context, docID string) ([]*websocket.History, error) {
	if !c.transport.IsOpen() {
		if err := c.transport.Open(); err != nil {
			return nil, fmt.Errorf("failed to open transport: %w", err)
		}
	}

	request := &proto.GetHistoryRequest{
		DocID: docID,
	}

	response, err := c.documentClient.GetHistory(ctx, request)
	if err != nil {
		return nil, fmt.Errorf("failed to get document history: %w", err)
	}

	if response == nil || response.History == nil {
		return nil, fmt.Errorf("received empty response")
	}

	history := make([]*websocket.History, len(response.History))
	for i, version := range response.History {
		timestamp, err := time.Parse(time.RFC3339, version.Timestamp)
		if err != nil {
			log.Warnf("failed to parse timestamp: %v, using current time", err)
			timestamp = time.Now()
		}

		updates := make([]int, len(version.Updates))
		for j, update := range version.Updates {
			updates[j] = int(update)
		}

		history[i] = &websocket.History{
			Updates:   updates,
			Version:   int(version.Version),
			Timestamp: timestamp,
		}
	}

	return history, nil
}

func (c *Client) Rollback(ctx context.Context, id string, version int) (*websocket.Document, error) {
	if !c.transport.IsOpen() {
		if err := c.transport.Open(); err != nil {
			return nil, fmt.Errorf("failed to open transport: %w", err)
		}
	}

	request := &proto.RollbackRequest{
		DocID:   id,
		Version: int32(version),
	}

	response, err := c.documentClient.Rollback(ctx, request)
	if err != nil {
		return nil, fmt.Errorf("failed to rollback document: %w", err)
	}

	if response == nil || response.Document == nil {
		return nil, fmt.Errorf("received empty response")
	}

	updates := make([]int, len(response.Document.Updates))
	for i, update := range response.Document.Updates {
		updates[i] = int(update)
	}

	timestamp, err := time.Parse(time.RFC3339, response.Document.Timestamp)
	if err != nil {
		log.Warnf("failed to parse timestamp: %v, using current time", err)
		timestamp = time.Now()
	}

	return &websocket.Document{
		ID:        response.Document.ID,
		Updates:   updates,
		Version:   int(response.Document.Version),
		Timestamp: timestamp,
	}, nil
}
