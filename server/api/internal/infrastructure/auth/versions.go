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

// VersionNotice is the body of POST /auth/versions: a version of a project's
// workflow was saved. The websocket server sends notices in no particular
// order, retries them, and sends a version's notice again when edits made
// during its save turn out to be in it, with more contributors and possibly
// the same LastEditedAt.
//
// Nothing is kept per version, so a repeated notice needs no deduplication:
// recording an edit only ever moves the project forward to LastEditedAt.
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

// VersionHandler serves POST /auth/versions: the project's updatedAt and
// updatedBy take the version's last edit when it is later than the project's.
//
// The request must carry the websocket server's shared secret in X-API-Secret.
// The /auth routes share the API's public listener, so without it anyone could
// set a project's last editor. With no secret configured, every notice is
// refused.
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
	// Never ahead of this clock. Project setters stamp time.Now(), so a later
	// metadata save would move a future updatedAt backwards, and the project
	// lock orders its changes by updatedAt.
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
