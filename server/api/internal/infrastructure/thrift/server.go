package thrift

import (
	"encoding/json"
	"io"
	"net/http"

	"github.com/apache/thrift/lib/go/thrift"
	"github.com/reearth/reearth-flow/api/proto"
	"github.com/reearth/reearthx/appx"
	"github.com/reearth/reearthx/log"
)

type ThriftJSONRequest struct {
	Method string          `json:"method"`
	Type   string          `json:"type"`
	SeqID  int             `json:"seqid"`
	Args   json.RawMessage `json:"args"`
}

type ThriftJSONResponse struct {
	Type  string          `json:"type"`
	SeqID int             `json:"seqid"`
	Value json.RawMessage `json:"value"`
}

type Server struct {
	processor        thrift.TProcessor
	handler          *AuthServiceHandler
	protocolFactory  thrift.TProtocolFactory
	transportFactory thrift.TTransportFactory
}

func NewServer(_ string, jwtProviders []appx.JWTProvider) *Server {
	handler := NewAuthServiceHandler(jwtProviders)
	processor := proto.NewAuthServiceProcessor(handler)

	protocolFactory := thrift.NewTJSONProtocolFactory()
	transportFactory := thrift.NewTTransportFactory()

	return &Server{
		processor:        processor,
		handler:          handler,
		protocolFactory:  protocolFactory,
		transportFactory: transportFactory,
	}
}

func (s *Server) Stop() {
}

func (s *Server) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	body, err := io.ReadAll(r.Body)
	if err != nil {
		log.Errorf("Error reading request body: %v", err)
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	var request ThriftJSONRequest
	if err := json.Unmarshal(body, &request); err != nil {
		log.Errorf("Error parsing JSON request: %v", err)
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	w.Header().Set("Content-Type", "application/json")

	if request.Method == "VerifyAPIToken" {
		var args struct {
			Request struct {
				Token string `json:"token"`
			} `json:"request"`
		}
		if err := json.Unmarshal(request.Args, &args); err != nil {
			log.Errorf("Error parsing request args: %v", err)
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}

		req := &proto.APITokenVerifyRequest{Token: args.Request.Token}
		resp, err := s.handler.VerifyAPIToken(r.Context(), req)
		if err != nil {
			log.Errorf("Error processing request: %v", err)
			http.Error(w, err.Error(), http.StatusInternalServerError)
			return
		}

		successValue := struct {
			Authorized bool `json:"authorized"`
		}{
			Authorized: resp.Authorized,
		}
		successJSON, err := json.Marshal(successValue)
		if err != nil {
			log.Errorf("Error marshaling success value: %v", err)
			http.Error(w, err.Error(), http.StatusInternalServerError)
			return
		}

		responseValue := struct {
			Success json.RawMessage `json:"success"`
		}{
			Success: successJSON,
		}
		responseValueJSON, err := json.Marshal(responseValue)
		if err != nil {
			log.Errorf("Error marshaling response value: %v", err)
			http.Error(w, err.Error(), http.StatusInternalServerError)
			return
		}

		response := ThriftJSONResponse{
			Type:  "REPLY",
			SeqID: request.SeqID,
			Value: responseValueJSON,
		}

		if err := json.NewEncoder(w).Encode(response); err != nil {
			log.Errorf("Error encoding response: %v", err)
			http.Error(w, err.Error(), http.StatusInternalServerError)
			return
		}
	} else {
		log.Errorf("Unknown method: %s", request.Method)
		http.Error(w, "Unknown method", http.StatusBadRequest)
		return
	}
}
