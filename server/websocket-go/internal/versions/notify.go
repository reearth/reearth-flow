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

// CallbackPath is the API endpoint told of every saved version, under the
// API's base URL (REEARTH_FLOW_THRIFT_AUTH_URL). It takes a Saved as JSON with
// the X-API-Secret header, and must be idempotent on (projectId, versionId):
// a delivery whose response was lost is sent again.
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
}

// NewNotifier returns a Notifier for cfg.
func NewNotifier(cfg NotifierConfig) *Notifier {
	n := &Notifier{
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

// Wait waits for deliveries in progress, or for ctx.
func (n *Notifier) Wait(ctx context.Context) error {
	done := make(chan struct{})
	go func() { n.wg.Wait(); close(done) }()
	select {
	case <-done:
		return nil
	case <-ctx.Done():
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
		time.Sleep(n.backoff(attempt))
	}
}

// post sends one delivery. retry reports whether a failure may succeed later:
// a request the API refused as malformed or unauthorised will not.
func (n *Notifier) post(body []byte) (retry bool, err error) {
	req, err := http.NewRequest(http.MethodPost, n.url, bytes.NewReader(body))
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
