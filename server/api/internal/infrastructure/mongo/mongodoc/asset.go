package mongodoc

import (
	"time"

	"github.com/reearth/reearth-flow/api/pkg/asset"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/account/accountdomain"
	"golang.org/x/exp/slices"
)

type AssetDocument struct {
	ID                      string
	Project                 *string // Made optional for workspace-based assets
	Workspace               string
	CreatedAt               time.Time
	User                    *string
	Integration             *string
	FileName                string
	Name                    string
	Size                    uint64
	URL                     string
	ContentType             string
	UUID                    string
	PreviewType             *string
	Thread                  *string
	ArchiveExtractionStatus *string
	FlatFiles               bool
	Public                  bool
	CoreSupport             bool
}

type AssetConsumer = Consumer[*AssetDocument, *asset.Asset]

func NewAssetConsumer(workspaces []accountdomain.WorkspaceID) *AssetConsumer {
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
		CoreSupport: asset.CoreSupport(),
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

	if pt := asset.PreviewType(); pt != nil {
		pts := pt.String()
		doc.PreviewType = &pts
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
	wid, err := accountdomain.WorkspaceIDFrom(d.Workspace)
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
		Public(d.Public).
		CoreSupport(d.CoreSupport)

	// Only set project if it exists
	if d.Project != nil {
		pid, err := id.ProjectIDFrom(*d.Project)
		if err != nil {
			return nil, err
		}
		b = b.Project(pid)
	}

	if d.User != nil {
		uid, err := accountdomain.UserIDFrom(*d.User)
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

	if d.PreviewType != nil {
		pt, ok := asset.PreviewTypeFrom(*d.PreviewType)
		if ok {
			b = b.Type(pt)
		}
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
