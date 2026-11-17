package versions

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"log/slog"
	"net/http"
	"strings"
	"sync"
	"time"
)

// CallbackPath is the API endpoint told of saved versions, under the API's base
// URL (REEARTH_FLOW_THRIFT_AUTH_URL). It receives a Saved as JSON.
//
// The API must refuse a request without the right X-API-Secret: the /auth
// routes share the API's public listener, and /auth/verify is open by design
// because its token is the credential. An open callback would let anyone set a
// project's last editor and update time.
//
// Notices arrive in no particular order and can arrive more than once: each is
// delivered and retried on its own. A version's notice is also sent again when
// edits made during the save turn out to be in it, with the attribution grown
// and a later LastEditedAt. The API should therefore apply a notice only when
// its LastEditedAt is later than what it already holds, which makes delivery
// order and repeats harmless; it must not drop a notice because it has seen
// its VersionID before.
const CallbackPath = "/auth/versions"

const (
	defaultNotifyAttempts = 8
	defaultNotifyTimeout  = 10 * time.Second
	notifyBackoffBase     = time.Second
	notifyBackoffMax      = 30 * time.Second
)

// NotifierConfig configures a Notifier.
type NotifierConfig struct {
	// APIURL is the API's base URL.
	APIURL string
	// Secret is sent as X-API-Secret. Empty sends none.
	Secret string
	// Client overrides the HTTP client (tests).
	Client *http.Client
	// Attempts bounds deliveries per version (default 8, about two minutes).
	Attempts int
	// Backoff returns the wait before retry n (n >= 1). Default: doubling from
	// one second, at most 30 seconds.
	Backoff func(n int) time.Duration
	Log     *slog.Logger
}

// Notifier tells the API about saved versions. Each delivery runs on its own
// goroutine, so saving a version never waits for the API; failed deliveries
// are retried with backoff.
type Notifier struct {
	url      string
	secret   string
	client   *http.Client
	attempts int
	backoff  func(int) time.Duration
	log      *slog.Logger
	wg       sync.WaitGroup
	// ctx bounds every delivery; Shutdown cancels it.
	ctx    context.Context
	cancel context.CancelFunc
}

// NewNotifier returns a Notifier for cfg.
func NewNotifier(cfg NotifierConfig) *Notifier {
	ctx, cancel := context.WithCancel(context.Background())
	n := &Notifier{
		ctx:      ctx,
		cancel:   cancel,
		url:      strings.TrimRight(cfg.APIURL, "/") + CallbackPath,
		secret:   cfg.Secret,
		client:   cfg.Client,
		attempts: cfg.Attempts,
		backoff:  cfg.Backoff,
		log:      cfg.Log,
	}
	if n.client == nil {
		n.client = &http.Client{Timeout: defaultNotifyTimeout}
	}
	if n.attempts <= 0 {
		n.attempts = defaultNotifyAttempts
	}
	if n.backoff == nil {
		n.backoff = defaultBackoff
	}
	if n.log == nil {
		n.log = slog.Default()
	}
	return n
}

func defaultBackoff(n int) time.Duration {
	d := notifyBackoffBase << (n - 1)
	if d <= 0 || d > notifyBackoffMax {
		return notifyBackoffMax
	}
	return d
}

// Notify starts delivering s and returns at once.
func (n *Notifier) Notify(s Saved) {
	n.wg.Add(1)
	go func() {
		defer n.wg.Done()
		n.deliver(s)
	}()
}

// Shutdown waits for deliveries in progress until ctx ends, then abandons the
// rest and waits for them to stop.
func (n *Notifier) Shutdown(ctx context.Context) error {
	done := make(chan struct{})
	go func() { n.wg.Wait(); close(done) }()
	select {
	case <-done:
		n.cancel()
		return nil
	case <-ctx.Done():
		n.cancel()
		<-done
		return ctx.Err()
	}
}

func (n *Notifier) deliver(s Saved) {
	body, err := json.Marshal(s)
	if err != nil {
		n.log.Error("encoding a version notice failed", "project", s.ProjectID, "version", s.VersionID, "err", err)
		return
	}
	for attempt := 1; ; attempt++ {
		retry, err := n.post(body)
		if err == nil {
			return
		}
		if !retry || attempt >= n.attempts {
			n.log.Error("telling the API about a version failed; giving up",
				"project", s.ProjectID, "version", s.VersionID, "attempts", attempt, "err", err)
			return
		}
		n.log.Warn("telling the API about a version failed; retrying",
			"project", s.ProjectID, "version", s.VersionID, "attempt", attempt, "err", err)
		select {
		case <-time.After(n.backoff(attempt)):
		case <-n.ctx.Done():
			n.log.Warn("telling the API about a version abandoned at shutdown",
				"project", s.ProjectID, "version", s.VersionID, "attempts", attempt)
			return
		}
	}
}

// post sends one delivery. retry reports whether a failure may succeed later:
// a request the API refused as malformed or unauthorised will not.
func (n *Notifier) post(body []byte) (retry bool, err error) {
	req, err := http.NewRequestWithContext(n.ctx, http.MethodPost, n.url, bytes.NewReader(body))
	if err != nil {
		return false, err
	}
	req.Header.Set("Content-Type", "application/json")
	if n.secret != "" {
		req.Header.Set("X-API-Secret", n.secret)
	}
	resp, err := n.client.Do(req)
	if err != nil {
		return true, err
	}
	_, _ = io.Copy(io.Discard, io.LimitReader(resp.Body, 4096))
	_ = resp.Body.Close()
	switch {
	case resp.StatusCode >= 200 && resp.StatusCode < 300:
		return false, nil
	case resp.StatusCode == http.StatusRequestTimeout, resp.StatusCode == http.StatusTooManyRequests,
		resp.StatusCode >= 500:
		return true, fmt.Errorf("api answered %d", resp.StatusCode)
	default:
		return false, fmt.Errorf("api answered %d", resp.StatusCode)
	}
}
