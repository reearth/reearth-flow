package server

import (
	"net/http/httptest"
	"testing"
	"time"

	gws "github.com/gorilla/websocket"
	"github.com/reearth/ygo/awareness"
	"github.com/reearth/ygo/crdt"
	"github.com/reearth/ygo/encoding"
	ygsync "github.com/reearth/ygo/sync"

	"github.com/reearth/reearth-flow/websocket-go/internal/docid"
)

// e2e_wire_test.go drives the real server with a real y-websocket client to
// prove the Go wire is byte-compatible with the browser's y-websocket: the
// sync handshake, an Update fan-out, and an Awareness update propagation.
//
// Wire framing (messageSync=0 / messageAwareness=1):
//
//	sync:      varUint(0) ‖ varUint(syncMsgType) ‖ <sync payload>
//	awareness: varUint(1) ‖ varBytes(awareness.EncodeUpdate)

const (
	wireMsgSync      = 0
	wireMsgAwareness = 1
)

// e2eClient is a minimal y-websocket client owning a local doc + awareness,
// mirroring a browser tab.
type e2eClient struct {
	t    *testing.T
	conn *gws.Conn
	doc  *crdt.Doc
	aw   *awareness.Awareness
}

func dialE2E(t *testing.T, ts *httptest.Server, room string, clientID uint64) *e2eClient {
	t.Helper()
	conn, _, err := gws.DefaultDialer.Dial(wsURL(ts, room), nil)
	if err != nil {
		t.Fatalf("dial %s: %v", room, err)
	}
	return &e2eClient{
		t:    t,
		conn: conn,
		doc:  crdt.New(crdt.WithClientID(crdt.ClientID(clientID))),
		aw:   awareness.New(clientID),
	}
}

func (c *e2eClient) close() { _ = c.conn.Close() }

func (c *e2eClient) writeSync(raw []byte) {
	enc := encoding.NewEncoder()
	enc.WriteVarUint(wireMsgSync)
	enc.WriteRaw(raw)
	if err := c.conn.WriteMessage(gws.BinaryMessage, enc.Bytes()); err != nil {
		c.t.Fatalf("write sync: %v", err)
	}
}

func (c *e2eClient) writeAwareness(update []byte) {
	enc := encoding.NewEncoder()
	enc.WriteVarUint(wireMsgAwareness)
	enc.WriteVarBytes(update)
	if err := c.conn.WriteMessage(gws.BinaryMessage, enc.Bytes()); err != nil {
		c.t.Fatalf("write awareness: %v", err)
	}
}

// readFrame reads one binary frame, returning the outer type and the raw sync
// message or VarBytes-unwrapped awareness update.
func (c *e2eClient) readFrame(timeout time.Duration) (uint64, []byte, error) {
	_ = c.conn.SetReadDeadline(time.Now().Add(timeout))
	_, data, err := c.conn.ReadMessage()
	_ = c.conn.SetReadDeadline(time.Time{})
	if err != nil {
		return 0, nil, err
	}
	dec := encoding.NewDecoder(data)
	mt, err := dec.ReadVarUint()
	if err != nil {
		return 0, nil, err
	}
	if mt == wireMsgAwareness {
		p, _ := dec.ReadVarBytes()
		return mt, p, nil
	}
	return mt, dec.RemainingBytes(), nil
}

// completeHandshake runs the y-websocket connect handshake: reply to the
// server's SyncStep1 and apply its SyncStep2.
func (c *e2eClient) completeHandshake() {
	// y-websocket also sends a client-initiated SyncStep1 on open.
	c.writeSync(ygsync.EncodeSyncStep1(c.doc))

	sawStep1, sawStep2 := false, false
	deadline := time.Now().Add(3 * time.Second)
	for (!sawStep1 || !sawStep2) && time.Now().Before(deadline) {
		mt, payload, err := c.readFrame(time.Second)
		if err != nil {
			continue
		}
		if mt != wireMsgSync {
			continue
		}
		smt, _, err := ygsync.ReadSyncMessage(payload)
		if err != nil {
			continue
		}
		switch smt {
		case ygsync.MsgSyncStep1:
			sawStep1 = true
			// EncodeSyncStep2 wants the full step-1 message, not the sub-payload.
			step2, err := ygsync.EncodeSyncStep2(c.doc, payload)
			if err != nil {
				c.t.Fatalf("EncodeSyncStep2: %v", err)
			}
			c.writeSync(step2)
		case ygsync.MsgSyncStep2:
			sawStep2 = true
			if _, err := ygsync.ApplySyncMessage(c.doc, payload, nil); err != nil {
				c.t.Fatalf("apply server SyncStep2: %v", err)
			}
		}
	}
	if !sawStep1 || !sawStep2 {
		c.t.Fatalf("handshake incomplete: step1=%v step2=%v", sawStep1, sawStep2)
	}
}

// TestE2EWireConformance is the canonical wire-compatibility acceptance gate.
func TestE2EWireConformance(t *testing.T) {
	cfg := testConfig()
	srv := New(cfg)
	// HandlerWithAPI(nil) is the production WS + /health surface without /api/*.
	ts := httptest.NewServer(srv.HandlerWithAPI(nil))
	defer ts.Close()

	const room = "550e8400-e29b-41d4-a716-446655440000"

	// ── Phase 1: two real clients complete the sync handshake ────────────────
	a := dialE2E(t, ts, room, 101)
	defer a.close()
	a.completeHandshake()

	b := dialE2E(t, ts, room, 202)
	defer b.close()
	b.completeHandshake()

	// ── Phase 2: an Update from A relays to B (server fan-out) ────────────────
	txt := a.doc.GetText("content")
	before := a.doc.StateVector()
	a.doc.Transact(func(txn *crdt.Transaction) {
		txt.Insert(txn, 0, "Hello from Client A", nil)
	})
	update := crdt.EncodeStateAsUpdateV1(a.doc, before)
	a.writeSync(ygsync.EncodeUpdate(update))

	if !waitForText(t, b, "content", "Hello from Client A") {
		t.Fatalf("client B never observed A's update — server did not relay the sync Update")
	}

	// ── Phase 3: an Awareness update from A propagates to B ───────────────────
	a.aw.SetLocalState(map[string]any{"user": map[string]any{"name": "alice"}})
	a.writeAwareness(a.aw.EncodeUpdate(nil))

	if !waitForAwareness(t, b, 101, "alice") {
		t.Fatalf("client B never observed A's awareness — server did not propagate Awareness")
	}
}

// waitForText drains frames until the client's named text equals want or times out.
func waitForText(t *testing.T, c *e2eClient, field, want string) bool {
	t.Helper()
	deadline := time.Now().Add(3 * time.Second)
	for time.Now().Before(deadline) {
		mt, payload, err := c.readFrame(500 * time.Millisecond)
		if err != nil {
			continue
		}
		if mt != wireMsgSync {
			continue
		}
		if _, err := ygsync.ApplySyncMessage(c.doc, payload, nil); err != nil {
			continue
		}
		if c.doc.GetText(field).ToString() == want {
			return true
		}
	}
	return false
}

// waitForAwareness drains frames until remoteID reports user.name == wantName.
func waitForAwareness(t *testing.T, c *e2eClient, remoteID uint64, wantName string) bool {
	t.Helper()
	deadline := time.Now().Add(3 * time.Second)
	for time.Now().Before(deadline) {
		mt, payload, err := c.readFrame(500 * time.Millisecond)
		if err != nil {
			continue
		}
		if mt != wireMsgAwareness {
			continue
		}
		if err := c.aw.ApplyUpdate(payload, nil); err != nil {
			continue
		}
		states := c.aw.GetStates()
		cs, ok := states[remoteID]
		if !ok {
			continue
		}
		user, _ := cs.State["user"].(map[string]any)
		if user != nil && user["name"] == wantName {
			return true
		}
	}
	return false
}

// TestE2EWireDocIDNormalization proves A on the bare id and B on "{id}:main"
// share a room over the real wire.
func TestE2EWireDocIDNormalization(t *testing.T) {
	if docid.Normalize("x:main") != docid.Normalize("x") {
		t.Fatalf("docid normalization precondition broken")
	}

	srv := New(testConfig())
	ts := httptest.NewServer(srv.HandlerWithAPI(nil))
	defer ts.Close()

	const id = "6ba7b810-9dad-11d1-80b4-00c04fd430c8"
	a := dialE2E(t, ts, id, 1)
	defer a.close()
	a.completeHandshake()

	b := dialE2E(t, ts, id+":main", 2)
	defer b.close()
	b.completeHandshake()

	txt := a.doc.GetText("content")
	before := a.doc.StateVector()
	a.doc.Transact(func(txn *crdt.Transaction) { txt.Insert(txn, 0, "unified", nil) })
	a.writeSync(ygsync.EncodeUpdate(crdt.EncodeStateAsUpdateV1(a.doc, before)))

	if !waitForText(t, b, "content", "unified") {
		t.Fatalf("bare-id and :main clients did not share a room over the real wire")
	}
}
