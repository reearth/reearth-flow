package auth

import (
	"context"
	"crypto/subtle"
	"encoding/json"
	"errors"
	"log"
	"net/http"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/rerror"
)

// VersionsPath receives the websocket server's notice of each saved version
// of a project's workflow.
const VersionsPath = "/auth/versions"

// maxVersionNoticeBytes bounds a notice body. A notice carries one version's
// contributor IDs, which the websocket server caps well below this.
const maxVersionNoticeBytes = 1 << 20

// VersionNotice is the body of POST /auth/versions, which the websocket server
// may send out of order, retry, or resend with more contributors and the same
// LastEditedAt. None of that needs deduplication, because recording an edit
// only ever moves the project forward.
type VersionNotice struct {
	SavedAt time.Time `json:"savedAt"`
	// LastEditedAt is when the version's last recorded edit was made.
	LastEditedAt time.Time `json:"lastEditedAt"`
	ProjectID    string    `json:"projectId"`
	Label        string    `json:"label"`
	// LastEditorID is absent when no recorded edit named its sender.
	LastEditorID   string   `json:"lastEditorId,omitempty"`
	ContributorIDs []string `json:"contributorIds"`
	VersionID      int64    `json:"versionId"`
}

// ProjectEditRecorder records an edit of a project made outside its own
// fields. repo.Project implements it.
type ProjectEditRecorder interface {
	RecordEdit(ctx context.Context, pid id.ProjectID, at time.Time, by *accountsid.UserID) error
}

// VersionHandler serves POST /auth/versions, moving the project's updatedAt and
// updatedBy to the version's last edit when it is later. It requires the
// websocket server's secret in X-API-Secret because the /auth routes share the
// public listener, and refuses every notice when no secret is configured.
type VersionHandler struct {
	projects ProjectEditRecorder
	now      func() time.Time
	secret   string
}

func NewVersionHandler(secret string, projects ProjectEditRecorder) *VersionHandler {
	return &VersionHandler{secret: secret, projects: projects, now: time.Now}
}

func (h *VersionHandler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}
	if !h.authorized(r) {
		http.Error(w, http.StatusText(http.StatusUnauthorized), http.StatusUnauthorized)
		return
	}

	var n VersionNotice
	if err := json.NewDecoder(http.MaxBytesReader(w, r.Body, maxVersionNoticeBytes)).Decode(&n); err != nil {
		http.Error(w, "invalid body", http.StatusBadRequest)
		return
	}
	pid, err := id.ProjectIDFrom(n.ProjectID)
	if err != nil {
		http.Error(w, "invalid projectId", http.StatusBadRequest)
		return
	}
	at := n.LastEditedAt
	if at.IsZero() {
		at = n.SavedAt
	}
	if at.IsZero() {
		http.Error(w, "lastEditedAt or savedAt required", http.StatusBadRequest)
		return
	}
	// Clamped to this clock: project setters stamp time.Now(), so a future
	// updatedAt would later move backwards, and the project lock orders by it.
	if now := h.now(); at.After(now) {
		at = now
	}
	var by *accountsid.UserID
	if n.LastEditorID != "" {
		uid, err := accountsid.UserIDFrom(n.LastEditorID)
		if err != nil {
			http.Error(w, "invalid lastEditorId", http.StatusBadRequest)
			return
		}
		by = &uid
	}

	if err := h.projects.RecordEdit(r.Context(), pid, at, by); err != nil {
		if errors.Is(err, rerror.ErrNotFound) {
			http.Error(w, "project not found", http.StatusNotFound)
			return
		}
		log.Printf("recording version %d of project %s failed: %v", n.VersionID, pid, err)
		http.Error(w, http.StatusText(http.StatusInternalServerError), http.StatusInternalServerError)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func (h *VersionHandler) authorized(r *http.Request) bool {
	if h.secret == "" {
		return false
	}
	got := r.Header.Get("X-API-Secret")
	return subtle.ConstantTimeCompare([]byte(got), []byte(h.secret)) == 1
}
