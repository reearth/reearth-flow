package thrift

import (
	"context"
	"net/http"
	"time"

	"github.com/apache/thrift/lib/go/thrift"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/websocket"
	"github.com/reearth/reearth-flow/api/proto"
	"github.com/reearth/reearthx/log"
)

type DocumentServiceHandler struct {
	client *websocket.Client
}

func NewDocumentServiceHandler(client *websocket.Client) *DocumentServiceHandler {
	return &DocumentServiceHandler{
		client: client,
	}
}

func (h *DocumentServiceHandler) GetLatest(ctx context.Context, request *proto.GetLatestRequest) (*proto.GetLatestResponse, error) {
	log.Infof("Handling GetLatest request for doc_id: %s", request.DocID)

	doc, err := h.client.GetLatest(ctx, request.DocID)
	if err != nil {
		log.Errorf("Failed to get latest document: %v", err)
		return nil, err
	}

	updates := make([]int32, len(doc.Updates()))
	for i, update := range doc.Updates() {
		updates[i] = int32(update)
	}

	timestamp := doc.Timestamp().Format(time.RFC3339)

	response := &proto.GetLatestResponse{
		Document: &proto.Document{
			ID:        doc.ID().String(),
			Updates:   updates,
			Version:   int32(doc.Version()),
			Timestamp: timestamp,
		},
	}

	return response, nil
}

func (h *DocumentServiceHandler) GetHistory(ctx context.Context, request *proto.GetHistoryRequest) (*proto.GetHistoryResponse, error) {
	log.Infof("Handling GetHistory request for doc_id: %s", request.DocID)

	history, err := h.client.GetHistory(ctx, request.DocID)
	if err != nil {
		log.Errorf("Failed to get document history: %v", err)
		return nil, err
	}

	historyItems := make([]*proto.History, len(history))
	for i, item := range history {
		updates := make([]int32, len(item.Updates()))
		for j, update := range item.Updates() {
			updates[j] = int32(update)
		}

		timestamp := item.Timestamp().Format(time.RFC3339)

		historyItems[i] = &proto.History{
			Updates:   updates,
			Version:   int32(item.Version()),
			Timestamp: timestamp,
		}
	}

	response := &proto.GetHistoryResponse{
		History: historyItems,
	}

	return response, nil
}

func (h *DocumentServiceHandler) Rollback(ctx context.Context, request *proto.RollbackRequest) (*proto.RollbackResponse, error) {
	log.Infof("Handling Rollback request for doc_id: %s to version: %d", request.DocID, request.Version)

	doc, err := h.client.Rollback(ctx, request.DocID, int(request.Version))
	if err != nil {
		log.Errorf("Failed to rollback document: %v", err)
		return nil, err
	}

	updates := make([]int32, len(doc.Updates()))
	for i, update := range doc.Updates() {
		updates[i] = int32(update)
	}

	timestamp := doc.Timestamp().Format(time.RFC3339)

	response := &proto.RollbackResponse{
		Document: &proto.Document{
			ID:        doc.ID().String(),
			Updates:   updates,
			Version:   int32(doc.Version()),
			Timestamp: timestamp,
		},
	}

	return response, nil
}

type DocumentServer struct {
	processor        thrift.TProcessor
	handler          *DocumentServiceHandler
	protocolFactory  thrift.TProtocolFactory
	transportFactory thrift.TTransportFactory
}

func NewDocumentServer(client *websocket.Client) *DocumentServer {
	handler := NewDocumentServiceHandler(client)
	processor := proto.NewDocumentServiceProcessor(handler)

	protocolFactory := thrift.NewTJSONProtocolFactory()
	transportFactory := thrift.NewTTransportFactory()

	return &DocumentServer{
		processor:        processor,
		handler:          handler,
		protocolFactory:  protocolFactory,
		transportFactory: transportFactory,
	}
}

func (s *DocumentServer) Handler() http.Handler {
	return http.HandlerFunc(thrift.NewThriftHandlerFunc(s.processor, s.protocolFactory, s.protocolFactory))
}
