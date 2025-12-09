package mongodoc

import (
	"time"

	"github.com/reearth/reearth-flow/api/pkg/asset"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"golang.org/x/exp/slices"
)

type AssetDocument struct {
	CreatedAt               time.Time
	Project                 *string // Made optional for workspace-based assets
	User                    *string
	Integration             *string
	Thread                  *string
	ArchiveExtractionStatus *string
	ID                      string
	Workspace               string
	FileName                string
	Name                    string
	URL                     string
	ContentType             string
	UUID                    string
	Size                    uint64
	FlatFiles               bool
	Public                  bool
}

type AssetConsumer = Consumer[*AssetDocument, *asset.Asset]

func NewAssetConsumer(workspaces []id.WorkspaceID) *AssetConsumer {
	return NewConsumer[*AssetDocument, *asset.Asset](func(a *asset.Asset) bool {
		return workspaces == nil || slices.Contains(workspaces, a.Workspace())
	})
}

func NewAsset(asset *asset.Asset) (*AssetDocument, string) {
	aid := asset.ID().String()
	doc := &AssetDocument{
		ID:          aid,
		Workspace:   asset.Workspace().String(),
		CreatedAt:   asset.CreatedAt(),
		FileName:    asset.FileName(),
		Name:        asset.Name(),
		Size:        asset.Size(),
		URL:         asset.URL(),
		ContentType: asset.ContentType(),
		UUID:        asset.UUID(),
		FlatFiles:   asset.FlatFiles(),
		Public:      asset.Public(),
	}

	// Only set project if it's not empty
	if pid := asset.Project(); !pid.IsNil() {
		pidStr := pid.String()
		doc.Project = &pidStr
	}

	if u := asset.User(); u != nil {
		uid := u.String()
		doc.User = &uid
	}

	if i := asset.Integration(); i != nil {
		iid := i.String()
		doc.Integration = &iid
	}

	if t := asset.Thread(); t != nil {
		tid := t.String()
		doc.Thread = &tid
	}

	if s := asset.ArchiveExtractionStatus(); s != nil {
		ss := s.String()
		doc.ArchiveExtractionStatus = &ss
	}

	return doc, aid
}

func (d *AssetDocument) Model() (*asset.Asset, error) {
	aid, err := id.AssetIDFrom(d.ID)
	if err != nil {
		return nil, err
	}
	wid, err := id.WorkspaceIDFrom(d.Workspace)
	if err != nil {
		return nil, err
	}

	b := asset.New().
		ID(aid).
		Workspace(wid).
		CreatedAt(d.CreatedAt).
		FileName(d.FileName).
		Name(d.Name).
		Size(d.Size).
		URL(d.URL).
		ContentType(d.ContentType).
		UUID(d.UUID).
		FlatFiles(d.FlatFiles).
		Public(d.Public)

	// Only set project if it exists
	if d.Project != nil {
		pid, err := id.ProjectIDFrom(*d.Project)
		if err != nil {
			return nil, err
		}
		b = b.Project(pid)
	}

	if d.User != nil {
		uid, err := id.UserIDFrom(*d.User)
		if err != nil {
			return nil, err
		}
		b = b.CreatedByUser(uid)
	} else if d.Integration != nil {
		iid, err := id.IntegrationIDFrom(*d.Integration)
		if err != nil {
			return nil, err
		}
		b = b.CreatedByIntegration(&iid)
	}

	if d.Thread != nil {
		tid, err := id.ThreadIDFrom(*d.Thread)
		if err != nil {
			return nil, err
		}
		b = b.Thread(&tid)
	}

	if d.ArchiveExtractionStatus != nil {
		s, ok := asset.ArchiveExtractionStatusFrom(*d.ArchiveExtractionStatus)
		if ok {
			b = b.ArchiveExtractionStatus(s)
		}
	}

	return b.Build()
}
