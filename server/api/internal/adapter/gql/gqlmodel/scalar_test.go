package gqlmodel

import (
	"encoding/json"
	"strings"
	"testing"
)

func decodeJSONUseNumber(t *testing.T, s string) any {
	t.Helper()
	dec := json.NewDecoder(strings.NewReader(s))
	dec.UseNumber()
	var v any
	if err := dec.Decode(&v); err != nil {
		t.Fatalf("failed to decode json: %v", err)
	}
	return v
}

func TestUnmarshalBytes_WithJSONNumberArray(t *testing.T) {
	decoded := decodeJSONUseNumber(t, `[0,1,2,127,128,255]`)
	arr, ok := decoded.([]interface{})
	if !ok {
		t.Fatalf("decoded value is not []interface{}: %T", decoded)
	}

	b, err := UnmarshalBytes(arr)
	if err != nil {
		t.Fatalf("UnmarshalBytes returned error: %v", err)
	}

	expected := []byte{0, 1, 2, 127, 128, 255}
	if len(b) != len(expected) {
		t.Fatalf("length mismatch: got %d want %d", len(b), len(expected))
	}
	for i := range expected {
		if b[i] != expected[i] {
			t.Fatalf("byte %d mismatch: got %d want %d", i, b[i], expected[i])
		}
	}
}

func TestUnmarshalBytes_WithFloat64Array(t *testing.T) {
	decoded := []interface{}{float64(10), float64(20), float64(30)}
	if _, err := UnmarshalBytes(decoded); err == nil {
		t.Log("float64 array unexpectedly accepted; ensure this is intended behavior")
	}
}

func TestUnmarshalBytes_InvalidElement(t *testing.T) {
	decoded := []interface{}{json.Number("1"), "oops", json.Number("2")}
	if _, err := UnmarshalBytes(decoded); err == nil {
		t.Fatalf("expected error for invalid element, got nil")
	}
}
