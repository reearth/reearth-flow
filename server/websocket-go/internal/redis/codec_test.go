package redis

import (
	"context"
	"testing"

	"github.com/alicebob/miniredis/v2"
	goredis "github.com/redis/go-redis/v9"
)

func newTestClient(t *testing.T) (*goredis.Client, *miniredis.Miniredis) {
	t.Helper()
	mr, err := miniredis.Run()
	if err != nil {
		t.Fatalf("miniredis.Run: %v", err)
	}
	t.Cleanup(mr.Close)
	c := goredis.NewClient(&goredis.Options{Addr: mr.Addr()})
	t.Cleanup(func() { _ = c.Close() })
	return c, mr
}

// TestXAddFieldOrderAndEncoding asserts the four XADD fields land in order
// type,data,clientId,timestamp with clientId and timestamp as decimal-ASCII.
func TestXAddFieldOrderAndEncoding(t *testing.T) {
	c, _ := newTestClient(t)
	ctx := context.Background()
	const clientID uint64 = 12345

	data := []byte{0x00, 0x01, 0xff}
	if err := xaddEntry(ctx, c, streamKey("proj1"), msgTypeSync, data, clientID); err != nil {
		t.Fatalf("xaddEntry: %v", err)
	}

	msgs, err := c.XRange(ctx, streamKey("proj1"), "-", "+").Result()
	if err != nil {
		t.Fatalf("XRange: %v", err)
	}
	if len(msgs) != 1 {
		t.Fatalf("got %d entries, want 1", len(msgs))
	}
	v := msgs[0].Values

	if v["type"] != "sync" {
		t.Errorf("type = %v, want sync", v["type"])
	}
	if got := []byte(v["data"].(string)); string(got) != string(data) {
		t.Errorf("data = %v, want %v", got, data)
	}
	if v["clientId"] != "12345" {
		t.Errorf("clientId = %v, want decimal-ASCII 12345", v["clientId"])
	}
	if ts, ok := v["timestamp"].(string); !ok || ts == "" {
		t.Errorf("timestamp = %v, want non-empty decimal-ASCII", v["timestamp"])
	} else {
		for _, r := range ts {
			if r < '0' || r > '9' {
				t.Errorf("timestamp %q is not decimal-ASCII", ts)
				break
			}
		}
	}
}

func TestXAddAwarenessType(t *testing.T) {
	c, _ := newTestClient(t)
	ctx := context.Background()
	if err := xaddEntry(ctx, c, streamKey("p"), msgTypeAwareness, []byte{1}, 7); err != nil {
		t.Fatalf("xaddEntry: %v", err)
	}
	msgs, _ := c.XRange(ctx, streamKey("p"), "-", "+").Result()
	if len(msgs) != 1 || msgs[0].Values["type"] != "awareness" {
		t.Fatalf("awareness type not written: %+v", msgs)
	}
}

// TestParseEntry exercises reader-side classification + self-filter: an entry
// whose clientId equals our own id is dropped; others are routed by type.
func TestParseEntry(t *testing.T) {
	const me uint64 = 999
	tests := []struct {
		name     string
		values   map[string]any
		wantKind streamKind
		wantData []byte
		wantSelf bool
	}{
		{"remote sync", map[string]any{"type": "sync", "data": "\x01\x02", "clientId": "1"}, kindSync, []byte{1, 2}, false},
		{"remote awareness", map[string]any{"type": "awareness", "data": "\x03", "clientId": "1"}, kindAwareness, []byte{3}, false},
		{"own entry filtered", map[string]any{"type": "sync", "data": "\x01", "clientId": "999"}, kindSync, []byte{1}, true},
		{"unknown type", map[string]any{"type": "other", "data": "\x01", "clientId": "1"}, kindUnknown, []byte{1}, false},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			e := parseEntry(goredis.XMessage{ID: "1-0", Values: tt.values}, me)
			if e.kind != tt.wantKind {
				t.Errorf("kind = %v, want %v", e.kind, tt.wantKind)
			}
			if e.isSelf != tt.wantSelf {
				t.Errorf("isSelf = %v, want %v", e.isSelf, tt.wantSelf)
			}
			if string(e.data) != string(tt.wantData) {
				t.Errorf("data = %v, want %v", e.data, tt.wantData)
			}
		})
	}
}
