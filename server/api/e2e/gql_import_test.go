package e2e

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"regexp"
	"sync/atomic"
	"testing"
)

type capturedImport struct {
	DocID string
	Body  []byte
}

type importRequest struct {
	Data []int `json:"data"`
}

func startMockWS(t *testing.T) (*httptest.Server, *atomic.Pointer[capturedImport]) {
	t.Helper()
	re := regexp.MustCompile(`^/api/document/([^/]+)/import$`)
	cap := &atomic.Pointer[capturedImport]{}
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			w.WriteHeader(http.StatusMethodNotAllowed)
			return
		}
		m := re.FindStringSubmatch(r.URL.Path)
		if len(m) != 2 {
			w.WriteHeader(http.StatusNotFound)
			return
		}
		b := make([]byte, r.ContentLength)
		_, _ = r.Body.Read(b)
		_ = r.Body.Close()
		cap.Store(&capturedImport{DocID: m[1], Body: b})
		w.WriteHeader(http.StatusOK)
	}))
	return srv, cap
}

func TestImportDocument_DirectToWebsocket_WithNumberArray(t *testing.T) {
	ws, cap := startMockWS(t)
	defer ws.Close()

	docID := "e2e-doc-id"
	payload := importRequest{Data: []int{1, 2, 3, 255}}
	body, _ := json.Marshal(payload)

	resp, err := http.Post(ws.URL+"/api/document/"+docID+"/import", "application/json", bytes.NewReader(body))
	if err != nil {
		t.Fatalf("post to mock websocket failed: %v", err)
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("unexpected status: %d", resp.StatusCode)
	}

	ci := cap.Load()
	if ci == nil {
		t.Fatalf("mock websocket did not capture request")
	}
	var got importRequest
	if err := json.Unmarshal(ci.Body, &got); err != nil {
		t.Fatalf("mock body unmarshal failed: %v body=%s", err, string(ci.Body))
	}
	if len(got.Data) != len(payload.Data) {
		t.Fatalf("length mismatch: got %d want %d", len(got.Data), len(payload.Data))
	}
	for i := range payload.Data {
		if got.Data[i] != payload.Data[i] {
			t.Fatalf("byte %d mismatch: got %d want %d", i, got.Data[i], payload.Data[i])
		}
	}
}
