package asset

import (
	"time"

	"github.com/reearth/reearth-flow/api/pkg/id"
)

type UploadBuilder struct {
	u *Upload
}

func NewUpload() *UploadBuilder {
	return &UploadBuilder{
		u: &Upload{},
	}
}

func (b *UploadBuilder) UUID(uuid string) *UploadBuilder {
	b.u.uuid = uuid
	return b
}

func (b *UploadBuilder) Workspace(workspace id.WorkspaceID) *UploadBuilder {
	b.u.workspace = workspace
	return b
}

func (b *UploadBuilder) FileName(fileName string) *UploadBuilder {
	b.u.fileName = fileName
	return b
}

func (b *UploadBuilder) ExpiresAt(expiresAt time.Time) *UploadBuilder {
	b.u.expiresAt = expiresAt
	return b
}

func (b *UploadBuilder) ContentLength(contentLength int64) *UploadBuilder {
	b.u.contentLength = contentLength
	return b
}

func (b *UploadBuilder) ContentType(contentType string) *UploadBuilder {
	b.u.contentType = contentType
	return b
}

func (b *UploadBuilder) ContentEncoding(contentEncoding string) *UploadBuilder {
	b.u.contentEncoding = contentEncoding
	return b
}

func (b *UploadBuilder) Build() *Upload {
	return b.u
}
