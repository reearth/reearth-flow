package http

import (
	"context"
	"errors"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"
)

type lockCall struct {
	room   string
	locked bool
	at     time.Time
}

type lockSignaler struct {
	calls []lockCall
	err   error
}

func (s *lockSignaler) SignalRollback(context.Context, string, bool) error { return nil }

func (s *lockSignaler) SignalLock(_ context.Context, room string, locked bool, at time.Time) error {
	s.calls = append(s.calls, lockCall{room: room, locked: locked, at: at})
	return s.err
}

func postLock(h http.Handler, path, body string) *httptest.ResponseRecorder {
	rec := httptest.NewRecorder()
	h.ServeHTTP(rec, httptest.NewRequest(http.MethodPost, path, strings.NewReader(body)))
	return rec
}

func TestLock_SignalsTheNormalizedRoom(t *testing.T) {
	t.Parallel()

	sig := &lockSignaler{}
	h := NewRouter(Deps{Store: &fakeStore{}, Signaler: sig})

	rec := postLock(h, "/api/document/proj1:main/lock", `{"locked":true,"changedAt":1700000000123}`)
	if rec.Code != http.StatusOK {
		t.Fatalf("status = %d, body = %s", rec.Code, rec.Body)
	}
	want := lockCall{room: "proj1", locked: true, at: time.UnixMilli(1700000000123)}
	if len(sig.calls) != 1 || sig.calls[0].room != want.room || sig.calls[0].locked != want.locked || !sig.calls[0].at.Equal(want.at) {
		t.Fatalf("calls = %+v, want [%+v]", sig.calls, want)
	}
}

func TestLock_RejectsABodyWithoutChangedAt(t *testing.T) {
	t.Parallel()

	sig := &lockSignaler{}
	h := NewRouter(Deps{Store: &fakeStore{}, Signaler: sig})

	if rec := postLock(h, "/api/document/proj1/lock", `{"locked":true}`); rec.Code != http.StatusBadRequest {
		t.Fatalf("status = %d, want 400", rec.Code)
	}
	if rec := postLock(h, "/api/document/proj1/lock", `not json`); rec.Code != http.StatusBadRequest {
		t.Fatalf("status = %d, want 400", rec.Code)
	}
	if len(sig.calls) != 0 {
		t.Fatalf("signaled on a bad request: %+v", sig.calls)
	}
}

func TestLock_ReportsASignalFailure(t *testing.T) {
	t.Parallel()

	h := NewRouter(Deps{Store: &fakeStore{}, Signaler: &lockSignaler{err: errors.New("boom")}})

	if rec := postLock(h, "/api/document/proj1/lock", `{"locked":false,"changedAt":1}`); rec.Code != http.StatusInternalServerError {
		t.Fatalf("status = %d, want 500", rec.Code)
	}
}

func TestLock_WithoutASignalerIsANoOp(t *testing.T) {
	t.Parallel()

	h := NewRouter(Deps{Store: &fakeStore{}})

	if rec := postLock(h, "/api/document/proj1/lock", `{"locked":true,"changedAt":1}`); rec.Code != http.StatusOK {
		t.Fatalf("status = %d, want 200", rec.Code)
	}
}
