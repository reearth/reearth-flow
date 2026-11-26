package redis

import (
	"context"
	"testing"
	"time"
)

// TestTakeOutboundGap_ReportsALossOnce: a lost update is reported once, so the
// room is republished once per loss.
func TestTakeOutboundGap_ReportsALossOnce(t *testing.T) {
	t.Parallel()
	_, mr := newTestClient(t)
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	const room = "heal-gap"

	relay := mustRelay(t, mr.Addr())
	relay.client.AddHook(&failXAdd{left: -1})
	t.Cleanup(func() { _ = relay.Close() })
	if err := relay.Start(ctx, &fakeSink{}); err != nil {
		t.Fatal(err)
	}
	relay.RoomActivated(room)
	if relay.TakeOutboundGap(room) {
		t.Fatal("a gap was reported before any update was lost")
	}

	publish(t, relay, room, 7)
	waitFor(t, 10*time.Second, func() bool { return relay.OutboundGap(room) })
	if !relay.TakeOutboundGap(room) {
		t.Fatal("the lost update was not reported")
	}
	if relay.OutboundGap(room) || relay.TakeOutboundGap(room) {
		t.Fatal("the gap was reported again after it was taken")
	}
}
