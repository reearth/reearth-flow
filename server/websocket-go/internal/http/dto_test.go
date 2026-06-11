package http

import (
	"encoding/json"
	"reflect"
	"testing"
	"time"
)

// TestUpdateBytesMarshalsAsIntArray pins the contract: the update blob MUST
// serialize as a JSON int array (0–255), NOT base64.
func TestUpdateBytesMarshalsAsIntArray(t *testing.T) {
	in := UpdateBytes{0x00, 0x01, 0xff}
	b, err := json.Marshal(in)
	if err != nil {
		t.Fatalf("marshal: %v", err)
	}
	if got, want := string(b), "[0,1,255]"; got != want {
		t.Fatalf("updates marshal = %s, want %s (NOT base64)", got, want)
	}
}

func TestUpdateBytesRoundTrip(t *testing.T) {
	want := UpdateBytes{0, 1, 2, 127, 128, 255}
	b, err := json.Marshal(want)
	if err != nil {
		t.Fatalf("marshal: %v", err)
	}
	var got UpdateBytes
	if err := json.Unmarshal(b, &got); err != nil {
		t.Fatalf("unmarshal: %v", err)
	}
	if !reflect.DeepEqual(got, want) {
		t.Fatalf("round-trip = %v, want %v", got, want)
	}
}

// TestUpdateBytesNilMarshalsAsEmptyArray ensures a nil/empty update is [] not null.
func TestUpdateBytesNilMarshalsAsEmptyArray(t *testing.T) {
	var in UpdateBytes
	b, err := json.Marshal(in)
	if err != nil {
		t.Fatalf("marshal: %v", err)
	}
	if got := string(b); got != "[]" {
		t.Fatalf("nil updates = %s, want []", got)
	}
}

func TestUpdateBytesRejectsOutOfRange(t *testing.T) {
	var got UpdateBytes
	if err := json.Unmarshal([]byte("[256]"), &got); err == nil {
		t.Fatalf("expected error unmarshaling out-of-range int 256")
	}
	if err := json.Unmarshal([]byte("[-1]"), &got); err == nil {
		t.Fatalf("expected error unmarshaling negative int -1")
	}
}

// TestDocumentResponseShape pins the JSON keys (id, timestamp, updates, version).
func TestDocumentResponseShape(t *testing.T) {
	ts := time.Date(2026, 6, 1, 12, 0, 0, 0, time.UTC)
	resp := DocumentResponse{
		ID:        "proj1",
		Timestamp: ts.Format(time.RFC3339),
		Updates:   UpdateBytes{1, 2, 3},
		Version:   42,
	}
	b, err := json.Marshal(resp)
	if err != nil {
		t.Fatalf("marshal: %v", err)
	}
	var raw map[string]json.RawMessage
	if err := json.Unmarshal(b, &raw); err != nil {
		t.Fatalf("unmarshal: %v", err)
	}
	if string(raw["id"]) != `"proj1"` {
		t.Fatalf("id = %s", raw["id"])
	}
	if string(raw["timestamp"]) != `"2026-06-01T12:00:00Z"` {
		t.Fatalf("timestamp = %s", raw["timestamp"])
	}
	if string(raw["updates"]) != "[1,2,3]" {
		t.Fatalf("updates = %s, want [1,2,3]", raw["updates"])
	}
	if string(raw["version"]) != "42" {
		t.Fatalf("version = %s, want 42", raw["version"])
	}
}

// TestImportRequestDecodesIntArray pins the import body {"data":[..ints..]}.
func TestImportRequestDecodesIntArray(t *testing.T) {
	var req ImportRequest
	if err := json.Unmarshal([]byte(`{"data":[0,1,255]}`), &req); err != nil {
		t.Fatalf("unmarshal: %v", err)
	}
	if !reflect.DeepEqual(req.Data, UpdateBytes{0, 1, 255}) {
		t.Fatalf("data = %v, want [0 1 255]", req.Data)
	}
}
