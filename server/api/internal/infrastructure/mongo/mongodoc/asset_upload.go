package mongodoc

import (
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/asset"
	"golang.org/x/exp/slices"
)

type AssetUploadDocument struct {
	UUID            string    `bson:"uuid"`
	Workspace       string    `bson:"workspace"`
	FileName        string    `bson:"filename"`
	ExpiresAt       time.Time `bson:"expires_at"`
	ContentLength   int64     `bson:"content_length"`
	ContentType     string    `bson:"content_type"`
	ContentEncoding string    `bson:"content_encoding"`
}

type AssetUploadConsumer = Consumer[*AssetUploadDocument, *asset.Upload]

func NewAssetUploadConsumer(workspaces []accountsid.WorkspaceID) *AssetUploadConsumer {
	return NewConsumer[*AssetUploadDocument, *asset.Upload](func(a *asset.Upload) bool {
		return workspaces == nil || slices.Contains(workspaces, a.Workspace())
	})
}

func NewAssetUpload(u *asset.Upload) (*AssetUploadDocument, string) {
	uuid := u.UUID()

	doc := &AssetUploadDocument{
		UUID:            uuid,
		Workspace:       u.Workspace().String(),
		FileName:        u.FileName(),
		ExpiresAt:       u.ExpiresAt(),
		ContentLength:   u.ContentLength(),
		ContentType:     u.ContentType(),
		ContentEncoding: u.ContentEncoding(),
	}
	return doc, uuid
}

func (d *AssetUploadDocument) Model() (*asset.Upload, error) {
	wid, err := accountsid.WorkspaceIDFrom(d.Workspace)
	if err != nil {
		return nil, err
	}

	return asset.NewUpload().
		UUID(d.UUID).
		Workspace(wid).
		FileName(d.FileName).
		ExpiresAt(d.ExpiresAt).
		ContentLength(d.ContentLength).
		ContentType(d.ContentType).
		ContentEncoding(d.ContentEncoding).
		Build(), nil
}
