package thrift

import (
	"context"
	"net/http"
	"time"

	"github.com/apache/thrift/lib/go/thrift"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/websocket"
	"github.com/reearth/reearth-flow/api/proto"
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
	doc, err := h.client.GetLatest(ctx, request.DocID)
	if err != nil {
		return nil, err
	}

	updates := make([]int32, len(doc.Updates))
	for i, update := range doc.Updates {
		updates[i] = int32(update)
	}

	timestamp := doc.Timestamp.Format(time.RFC3339)

	response := &proto.GetLatestResponse{
		Document: &proto.Document{
			ID:        doc.ID,
			Updates:   updates,
			Version:   int32(doc.Version),
			Timestamp: timestamp,
		},
	}

	return response, nil
}

func (h *DocumentServiceHandler) GetHistory(ctx context.Context, request *proto.GetHistoryRequest) (*proto.GetHistoryResponse, error) {
	history, err := h.client.GetHistory(ctx, request.DocID)
	if err != nil {
		return nil, err
	}

	historyItems := make([]*proto.History, len(history))
	for i, item := range history {
		updates := make([]int32, len(item.Updates))
		for j, update := range item.Updates {
			updates[j] = int32(update)
		}

		timestamp := item.Timestamp.Format(time.RFC3339)

		historyItems[i] = &proto.History{
			Updates:   updates,
			Version:   int32(item.Version),
			Timestamp: timestamp,
		}
	}

	response := &proto.GetHistoryResponse{
		History: historyItems,
	}

	return response, nil
}

func (h *DocumentServiceHandler) Rollback(ctx context.Context, request *proto.RollbackRequest) (*proto.RollbackResponse, error) {
	doc, err := h.client.Rollback(ctx, request.DocID, int(request.Version))
	if err != nil {
		return nil, err
	}

	updates := make([]int32, len(doc.Updates))
	for i, update := range doc.Updates {
		updates[i] = int32(update)
	}

	timestamp := doc.Timestamp.Format(time.RFC3339)

	response := &proto.RollbackResponse{
		Document: &proto.Document{
			ID:        doc.ID,
			Updates:   updates,
			Version:   int32(doc.Version),
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
