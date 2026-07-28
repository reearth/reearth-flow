package gcs

import (
	"context"
	"encoding/hex"
	"testing"

	"github.com/reearth/ygo/crdt"
)

// Guards the reearth-flow persistence layer: a nested `position` Y.Map inside a
// workflow node must survive the gcs adapter's store->load cycle (V2 snapshot
// persist + V1 serve). The yjs<->ygo library-level interop guarantee this
// depends on lives in the ygo repo (crdt/nested_map_yjs_interop_test.go); this
// test covers only the adapter round-trip that is specific to this repo.
//
// The fixture is genuine yjs@13.6.31 V1 output for:
//
//	root map "workflows" -> "wf-1" -> "nodes"
//	    -> "nodeA".position = {x:100, y:200}
//	    -> "nodeB".position = {x:100.5, y:-200.25}
const fxNestedPositionV1 = "010baed5e49b0500270109776f726b666c6f77730477662d31012700aed5e49b0500056e6f646573012700aed5e49b0501056e6f646541012700aed5e49b050208706f736974696f6e012800aed5e49b05030178017da4012800aed5e49b05030179017d88032700aed5e49b0501056e6f646542012800aed5e49b05060269640177056e6f6465422700aed5e49b050608706f736974696f6e012800aed5e49b05080178017c42c900002800aed5e49b05080179017cc348400000"

func mustHexFixture(t *testing.T, s string) []byte {
	t.Helper()
	b, err := hex.DecodeString(s)
	if err != nil {
		t.Fatalf("decode fixture hex: %v", err)
	}
	return b
}

// nestedPositionOf navigates workflows -> wf-1 -> nodes -> <node> -> position
// via Entries() (YMap.Get does not unwrap nested shared types).
func nestedPositionOf(t *testing.T, doc *crdt.Doc, node string) map[string]any {
	t.Helper()
	root := doc.GetMap("workflows").Entries()
	wf, ok := root["wf-1"].(map[string]any)
	if !ok {
		t.Fatalf("workflows[wf-1] missing or not a map: %#v", root["wf-1"])
	}
	nodes, ok := wf["nodes"].(map[string]any)
	if !ok {
		t.Fatalf("wf-1[nodes] missing or not a map: %#v", wf["nodes"])
	}
	nb, ok := nodes[node].(map[string]any)
	if !ok {
		t.Fatalf("nodes[%s] missing or not a map: %#v", node, nodes[node])
	}
	pos, _ := nb["position"].(map[string]any)
	return pos
}

func nestedFloat(t *testing.T, m map[string]any, key string) float64 {
	t.Helper()
	switch n := m[key].(type) {
	case float64:
		return n
	case float32:
		return float64(n)
	case int:
		return float64(n)
	case int64:
		return float64(n)
	case uint64:
		return float64(n)
	default:
		t.Fatalf("position[%s] missing or not numeric: %#v (%T)", key, m[key], m[key])
		return 0
	}
}

// TestGCSStoreLoad_PreservesNestedPositionMap feeds the adapter a real yjs V1
// update, then asserts Load reconstructs the nested position map intact.
func TestGCSStoreLoad_PreservesNestedPositionMap(t *testing.T) {
	client, bucket := newFakeGCS(t)
	a, err := New(Options{Client: client, Bucket: bucket, Locker: NewNoLock()})
	if err != nil {
		t.Fatalf("gcs.New: %v", err)
	}

	ctx := context.Background()
	const room = "01ktn5as6ndekfx7xk02dmxs63"
	if _, err := a.AppendUpdate(ctx, room, mustHexFixture(t, fxNestedPositionV1)); err != nil {
		t.Fatalf("AppendUpdate: %v", err)
	}

	lr, err := a.Load(ctx, room)
	if err != nil {
		t.Fatalf("Load: %v", err)
	}
	if len(lr.Update) == 0 {
		t.Fatal("Load returned empty state")
	}

	doc := crdt.New()
	if err := doc.ApplyUpdate(lr.Update); err != nil {
		t.Fatalf("ApplyUpdate(loaded): %v", err)
	}

	pos := nestedPositionOf(t, doc, "nodeB")
	if len(pos) == 0 {
		t.Fatal("nodeB.position is empty after store->load (nested-map data loss)")
	}
	if x, y := nestedFloat(t, pos, "x"), nestedFloat(t, pos, "y"); x != 100.5 || y != -200.25 {
		t.Fatalf("nodeB.position wrong: got x=%v y=%v, want x=100.5 y=-200.25", x, y)
	}
	if sib := nestedPositionOf(t, doc, "nodeA"); len(sib) == 0 {
		t.Fatal("sibling nodeA.position empty after store->load")
	}
}
