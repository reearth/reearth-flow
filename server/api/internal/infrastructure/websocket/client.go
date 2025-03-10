package websocket

import (
	"context"
	"fmt"
	"time"

	"github.com/apache/thrift/lib/go/thrift"
	"github.com/reearth/reearth-flow/api/pkg/document"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/proto"
	"github.com/reearth/reearthx/log"
)

type WebsocketConfig struct {
	ServerURL string `json:"server_url"`
}

type WebsocketClient interface {
	GetLatest(ctx context.Context, docID string) (*document.Document, error)
	GetHistory(ctx context.Context, docID string) ([]*document.History, error)
	Rollback(ctx context.Context, docID string, version int) (*document.Document, error)
	Close() error
}

type ThriftClient interface {
	GetLatest(ctx context.Context, request *proto.GetLatestRequest) (*proto.GetLatestResponse, error)
	GetHistory(ctx context.Context, request *proto.GetHistoryRequest) (*proto.GetHistoryResponse, error)
	Rollback(ctx context.Context, request *proto.RollbackRequest) (*proto.RollbackResponse, error)
}

type WebsocketRepo struct {
	config    WebsocketConfig
	client    ThriftClient
	transport thrift.TTransport
}

func NewDocumentRepo(ctx context.Context, config WebsocketConfig) (WebsocketClient, error) {
	log.Debugfc(ctx, "document: initializing document service client with serverURL=%s", config.ServerURL)

	if config.ServerURL == "" {
		config.ServerURL = "http://localhost:8000"
		log.Debugfc(ctx, "document: using default serverURL=%s", config.ServerURL)
	}

	docEndpoint := config.ServerURL + "/doc"
	trans, err := thrift.NewTHttpClient(docEndpoint)
	if err != nil {
		log.Errorfc(ctx, "document: failed to create Thrift HTTP client: %v", err)
		return nil, fmt.Errorf("failed to create Thrift HTTP client: %w", err)
	}

	cfg := &thrift.TConfiguration{}
	framedTransport := thrift.NewTFramedTransportConf(trans, cfg)

	protocolFactory := thrift.NewTBinaryProtocolFactoryConf(cfg)

	documentClient := proto.NewDocumentServiceClientFactory(framedTransport, protocolFactory)

	log.Debugfc(ctx, "document: successfully created document service client")
	return &WebsocketRepo{
		config:    config,
		client:    documentClient,
		transport: framedTransport,
	}, nil
}

func (d *WebsocketRepo) Close() error {
	log.Debug("document: closing client connection")
	return d.transport.Close()
}

func (d *WebsocketRepo) ensureConnection() error {
	if !d.transport.IsOpen() {
		log.Debug("document: opening transport connection")
		if err := d.transport.Open(); err != nil {
			log.Errorf("document: failed to open transport: %v", err)
			return fmt.Errorf("failed to open transport: %w", err)
		}
	}
	return nil
}

func (d *WebsocketRepo) GetLatest(ctx context.Context, docID string) (*document.Document, error) {
	log.Debugfc(ctx, "document: getting latest document for docID=%s", docID)
	if err := d.ensureConnection(); err != nil {
		return nil, err
	}

	request := &proto.GetLatestRequest{
		DocID: docID,
	}

	response, err := d.client.GetLatest(ctx, request)
	if err != nil {
		log.Errorfc(ctx, "document: failed to get latest document: %v", err)
		return nil, fmt.Errorf("failed to get latest document: %w", err)
	}

	if response == nil || response.Document == nil {
		log.Errorfc(ctx, "document: received empty response for docID=%s", docID)
		return nil, fmt.Errorf("received empty response")
	}

	doc, err := convertProtoToDocument(response.Document)
	if err != nil {
		return nil, err
	}

	log.Debugfc(ctx, "document: successfully retrieved latest document with ID=%s version=%d", 
		doc.ID(), doc.Version())
	return doc, nil
}

func (d *WebsocketRepo) GetHistory(ctx context.Context, docID string) ([]*document.History, error) {
	log.Debugfc(ctx, "document: getting history for docID=%s", docID)
	if err := d.ensureConnection(); err != nil {
		return nil, err
	}

	request := &proto.GetHistoryRequest{
		DocID: docID,
	}

	response, err := d.client.GetHistory(ctx, request)
	if err != nil {
		log.Errorfc(ctx, "document: failed to get document history: %v", err)
		return nil, fmt.Errorf("failed to get document history: %w", err)
	}

	if response == nil || response.History == nil {
		log.Errorfc(ctx, "document: received empty history response for docID=%s", docID)
		return nil, fmt.Errorf("received empty response")
	}

	history := make([]*document.History, len(response.History))
	for i, version := range response.History {
		updates := make([]int, len(version.Updates))
		for j, update := range version.Updates {
			updates[j] = int(update)
		}

		timestamp, err := time.Parse(time.RFC3339, version.Timestamp)
		if err != nil {
			log.Warnfc(ctx, "document: failed to parse timestamp: %v, using current time", err)
			timestamp = time.Now()
		}

		history[i] = document.NewHistory(updates, int(version.Version), timestamp)
	}

	log.Debugfc(ctx, "document: successfully retrieved %d history entries for docID=%s", 
		len(history), docID)
	return history, nil
}

func (d *WebsocketRepo) Rollback(ctx context.Context, docID string, version int) (*document.Document, error) {
	log.Debugfc(ctx, "document: rolling back docID=%s to version=%d", docID, version)
	if err := d.ensureConnection(); err != nil {
		return nil, err
	}

	request := &proto.RollbackRequest{
		DocID:   docID,
		Version: int32(version),
	}

	response, err := d.client.Rollback(ctx, request)
	if err != nil {
		log.Errorfc(ctx, "document: failed to rollback document: %v", err)
		return nil, fmt.Errorf("failed to rollback document: %w", err)
	}

	if response == nil || response.Document == nil {
		log.Errorfc(ctx, "document: received empty rollback response for docID=%s", docID)
		return nil, fmt.Errorf("received empty response")
	}

	doc, err := convertProtoToDocument(response.Document)
	if err != nil {
		return nil, err
	}

	log.Debugfc(ctx, "document: successfully rolled back document to version=%d", doc.Version())
	return doc, nil
}

func convertProtoToDocument(protoDoc *proto.Document) (*document.Document, error) {
	updates := make([]int, len(protoDoc.Updates))
	for i, update := range protoDoc.Updates {
		updates[i] = int(update)
	}

	timestamp, err := time.Parse(time.RFC3339, protoDoc.Timestamp)
	if err != nil {
		log.Warnf("document: failed to parse timestamp: %v, using current time", err)
		timestamp = time.Now()
	}

	docID, err := id.DocumentIDFrom(protoDoc.ID)
	if err != nil {
		log.Errorf("document: failed to parse document ID: %v", err)
		return nil, fmt.Errorf("failed to parse document ID: %w", err)
	}

	return document.NewDocument(docID, updates, int(protoDoc.Version), timestamp), nil
}
