package asset

import (
	"time"

	"github.com/reearth/reearth-flow/api/pkg/id"
)

type Upload struct {
	uuid            string
	workspace       id.WorkspaceID
	fileName        string
	expiresAt       time.Time
	contentLength   int64
	contentType     string
	contentEncoding string
}

func (u *Upload) UUID() string {
	return u.uuid
}

func (u *Upload) Workspace() id.WorkspaceID {
	return u.workspace
}

func (u *Upload) FileName() string {
	return u.fileName
}

func (u *Upload) ExpiresAt() time.Time {
	return u.expiresAt
}

func (u *Upload) Expired(t time.Time) bool {
	return t.After(u.expiresAt)
}

func (u *Upload) ContentLength() int64 {
	return u.contentLength
}

func (u *Upload) ContentType() string {
	return u.contentType
}

func (u *Upload) ContentEncoding() string {
	return u.contentEncoding
}
