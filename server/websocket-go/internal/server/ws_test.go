package server

import (
	"context"
	"net/http"
	"net/http/httptest"
	"strings"
	"sync/atomic"
	"testing"
	"time"

	gws "github.com/gorilla/websocket"
	"github.com/reearth/ygo/crdt"
	"github.com/reearth/ygo/encoding"
	ygsync "github.com/reearth/ygo/sync"

	"github.com/reearth/reearth-flow/websocket-go/internal/config"
)

func testConfig() *config.Config {
	return &config.Config{
		Origins:         []string{"*"},
		WSPort:          0,
		AppEnv:          "test",
		RedisURL:        "redis://127.0.0.1:6379",
		MaxConnections:  10000,
		MaxPeersPerRoom: 256,
		MaxRooms:        50000,
	}
}

// TestNewPropagatesDoSCaps verifies New sets finite, non-zero caps (ygo defaults to 0 = unlimited).
func TestNewPropagatesDoSCaps(t *testing.T) {
	cfg := &config.Config{
		Origins:         []string{"*"},
		MaxConnections:  1234,
		MaxPeersPerRoom: 56,
		MaxRooms:        7890,
	}
	p := New(cfg).WSProvider()
	if p.MaxConnections != 1234 {
		t.Errorf("MaxConnections = %d, want 1234", p.MaxConnections)
	}
	if p.MaxPeersPerRoom != 56 {
		t.Errorf("MaxPeersPerRoom = %d, want 56", p.MaxPeersPerRoom)
	}
	if p.MaxRooms != 7890 {
		t.Errorf("MaxRooms = %d, want 7890", p.MaxRooms)
	}
	if p.MaxConnections == 0 || p.MaxPeersPerRoom == 0 || p.MaxRooms == 0 {
		t.Fatalf("a cap is 0 (unlimited): conns=%d peers=%d rooms=%d",
			p.MaxConnections, p.MaxPeersPerRoom, p.MaxRooms)
	}
}

func wsURL(ts *httptest.Server, room string) string {
	return "ws" + strings.TrimPrefix(ts.URL, "http") + "/" + room
}

// readOne reads one binary WS frame, returning the outer message type and the
// remaining payload (sync payload is raw; awareness is VarBytes-wrapped).
func readOne(t *testing.T, conn *gws.Conn) (uint64, []byte) {
	t.Helper()
	_ = conn.SetReadDeadline(time.Now().Add(2 * time.Second))
	_, data, err := conn.ReadMessage()
	_ = conn.SetReadDeadline(time.Time{})
	if err != nil {
		t.Fatalf("read: %v", err)
	}
	dec := encoding.NewDecoder(data)
	mt, err := dec.ReadVarUint()
	if err != nil {
		t.Fatalf("decode type: %v", err)
	}
	if mt == 1 {
		p, _ := dec.ReadVarBytes()
		return mt, p
	}
	return mt, dec.RemainingBytes()
}

// TestSyncHandshake verifies a yjs client completes the SyncStep1 -> SyncStep2
// handshake against the mounted server.
func TestSyncHandshake(t *testing.T) {
	srv := New(testConfig())
	ts := httptest.NewServer(srv.Handler())
	defer ts.Close()

	conn, _, err := gws.DefaultDialer.Dial(wsURL(ts, "550e8400-e29b-41d4-a716-446655440000"), nil)
	if err != nil {
		t.Fatalf("dial: %v", err)
	}
	defer conn.Close()

	doc := crdt.New()
	sawStep1, sawStep2 := false, false
	for i := 0; i < 3; i++ {
		mt, payload := readOne(t, conn)
		if mt != 0 {
			continue // awareness frame
		}
		mtype, _, err := ygsync.ReadSyncMessage(payload)
		if err != nil {
			t.Fatalf("ReadSyncMessage: %v", err)
		}
		switch mtype {
		case ygsync.MsgSyncStep1:
			sawStep1 = true
			step2, err := ygsync.EncodeSyncStep2(doc, payload)
			if err != nil {
				t.Fatalf("EncodeSyncStep2: %v", err)
			}
			enc := encoding.NewEncoder()
			enc.WriteVarUint(0)
			enc.WriteRaw(step2)
			if err := conn.WriteMessage(gws.BinaryMessage, enc.Bytes()); err != nil {
				t.Fatalf("write step2: %v", err)
			}
		case ygsync.MsgSyncStep2:
			sawStep2 = true
			_, _ = ygsync.ApplySyncMessage(doc, payload, nil)
		}
	}
	if !sawStep1 || !sawStep2 {
		t.Fatalf("handshake incomplete: step1=%v step2=%v", sawStep1, sawStep2)
	}
}

// TestDocIDNormalizationRoutesToSameRoom verifies "/{uuid}:main" and "/{uuid}"
// share a room: a write from a :main peer is observed by a bare-id peer.
func TestDocIDNormalizationRoutesToSameRoom(t *testing.T) {
	srv := New(testConfig())
	ts := httptest.NewServer(srv.Handler())
	defer ts.Close()

	const id = "550e8400-e29b-41d4-a716-446655440000"

	// Peer A on the bare id.
	connA, _, err := gws.DefaultDialer.Dial(wsURL(ts, id), nil)
	if err != nil {
		t.Fatalf("dial A: %v", err)
	}
	defer connA.Close()
	docA := crdt.New()
	for i := 0; i < 3; i++ {
		mt, payload := readOne(t, connA)
		if mt == 0 {
			_, _ = ygsync.ApplySyncMessage(docA, payload, nil)
		}
	}

	// Peer B on "{id}:main".
	connB, _, err := gws.DefaultDialer.Dial(wsURL(ts, id+":main"), nil)
	if err != nil {
		t.Fatalf("dial B: %v", err)
	}
	defer connB.Close()
	docB := crdt.New()
	for i := 0; i < 3; i++ {
		mt, payload := readOne(t, connB)
		if mt == 0 {
			_, _ = ygsync.ApplySyncMessage(docB, payload, nil)
		}
	}

	txt := docB.GetText("t")
	before := docB.StateVector()
	docB.Transact(func(txn *crdt.Transaction) {
		txt.Insert(txn, 0, "hello", nil)
	})
	update := crdt.EncodeStateAsUpdateV1(docB, before)
	enc := encoding.NewEncoder()
	enc.WriteVarUint(0)
	enc.WriteVarUint(uint64(ygsync.MsgUpdate))
	enc.WriteVarBytes(update)
	if err := connB.WriteMessage(gws.BinaryMessage, enc.Bytes()); err != nil {
		t.Fatalf("write update: %v", err)
	}

	deadline := time.Now().Add(3 * time.Second)
	got := false
	for time.Now().Before(deadline) {
		_ = connA.SetReadDeadline(time.Now().Add(time.Second))
		_, data, err := connA.ReadMessage()
		if err != nil {
			continue
		}
		dec := encoding.NewDecoder(data)
		mt, _ := dec.ReadVarUint()
		if mt == 0 {
			if _, e := ygsync.ApplySyncMessage(docA, dec.RemainingBytes(), nil); e == nil {
				if docA.GetText("t").ToString() == "hello" {
					got = true
					break
				}
			}
		}
	}
	if !got {
		t.Fatalf("peer A on bare id did not observe peer B's write on :main — rooms not unified")
	}
}

// TestPeriodicSyncTicks verifies the re-sync loop runs at the configured interval.
func TestPeriodicSyncTicks(t *testing.T) {
	srv := New(testConfig())
	var ticks int64
	srv.onPeriodicSync = func() { atomic.AddInt64(&ticks, 1) }

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	go srv.StartPeriodicSync(ctx, 20*time.Millisecond)

	deadline := time.Now().Add(2 * time.Second)
	for time.Now().Before(deadline) {
		if atomic.LoadInt64(&ticks) >= 3 {
			return
		}
		time.Sleep(10 * time.Millisecond)
	}
	t.Fatalf("periodic sync did not tick >=3 times, got %d", atomic.LoadInt64(&ticks))
}

func TestPeriodicSyncDefaultInterval(t *testing.T) {
	if defaultResyncInterval != 30*time.Second {
		t.Fatalf("defaultResyncInterval = %v, want 30s", defaultResyncInterval)
	}
}

var _ http.Handler = New(testConfig()).Handler()
