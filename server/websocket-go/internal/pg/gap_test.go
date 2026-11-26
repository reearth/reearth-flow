package pg

import (
	"testing"
	"time"
)

// TestTakeOutboundGap_ReportsALossOnce: a lost update is reported once, so the
// room is republished once per loss.
func TestTakeOutboundGap_ReportsALossOnce(t *testing.T) {
	t.Parallel()
	pool := newPool(t)

	relay := startRelayOn(t, &faultyQ{Pool: pool, failLeft: -1}, &fakeSink{})
	relay.RoomActivated(deliveryRoom)
	if relay.TakeOutboundGap(deliveryRoom) {
		t.Fatal("a gap was reported before any update was lost")
	}

	publishSync(t, relay, 7)
	eventually(t, 8*time.Second, "the room to be flagged", func() bool { return relay.OutboundGap(deliveryRoom) })
	if !relay.TakeOutboundGap(deliveryRoom) {
		t.Fatal("the lost update was not reported")
	}
	if relay.OutboundGap(deliveryRoom) || relay.TakeOutboundGap(deliveryRoom) {
		t.Fatal("the gap was reported again after it was taken")
	}
}
