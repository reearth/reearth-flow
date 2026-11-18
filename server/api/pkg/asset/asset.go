package asset

import (
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/account/accountdomain"
	reearthxasset "github.com/reearth/reearthx/asset/domain/asset"
)

// WorkspaceOnlyProjectID marks workspace-only assets, because reearthx requires
// every asset to have a project ID.
var WorkspaceOnlyProjectID = func() id.ProjectID {
	return id.NewProjectID()
}()

type (
	Asset                   = AssetWrapper
	ArchiveExtractionStatus = reearthxasset.ArchiveExtractionStatus
	SortType                = reearthxasset.SortType
)

const (
	ArchiveExtractionStatusPending    = reearthxasset.ArchiveExtractionStatusPending
	ArchiveExtractionStatusInProgress = reearthxasset.ArchiveExtractionStatusInProgress
	ArchiveExtractionStatusDone       = reearthxasset.ArchiveExtractionStatusDone
	ArchiveExtractionStatusFailed     = reearthxasset.ArchiveExtractionStatusFailed
	ArchiveExtractionStatusSkipped    = reearthxasset.ArchiveExtractionStatusSkipped
)

var (
	SortTypeID   = reearthxasset.SortTypeID
	SortTypeNAME = reearthxasset.SortTypeName
	SortTypeSIZE = reearthxasset.SortTypeSize
	SortTypeDATE = SortType{Key: "date"}
)

var (
	ArchiveExtractionStatusFrom = reearthxasset.ArchiveExtractionStatusFrom
)

func NewID() id.AssetID {
	return id.NewAssetID()
}

func NewProjectID() id.ProjectID {
	return id.NewProjectID()
}

// Builder creates a new asset using reearthx's builder
type Builder struct {
	rxBuilder     *reearthxasset.Builder
	projectID     *id.ProjectID
	threadID      *id.ThreadID
	integrationID *id.IntegrationID
}

func New() *Builder {
	return &Builder{
		rxBuilder: reearthxasset.New(),
	}
}

func (b *Builder) ID(v id.AssetID) *Builder {
	b.rxBuilder = b.rxBuilder.ID(ConvertAssetIDToReearthx(v))
	return b
}

func (b *Builder) NewID() *Builder {
	b.rxBuilder = b.rxBuilder.NewID()
	return b
}

func (b *Builder) Project(v id.ProjectID) *Builder {
	b.projectID = &v
	b.rxBuilder = b.rxBuilder.Project(ConvertProjectIDToReearthx(v))
	return b
}

func (b *Builder) Workspace(v accountsid.WorkspaceID) *Builder {
	// TODO: after migration, remove this cast
	b.rxBuilder = b.rxBuilder.Workspace(accountdomain.WorkspaceID(v))
	if b.projectID == nil {
		b.projectID = &WorkspaceOnlyProjectID
		b.rxBuilder = b.rxBuilder.Project(ConvertProjectIDToReearthx(WorkspaceOnlyProjectID))
	}
	return b
}

func (b *Builder) CreatedAt(v time.Time) *Builder {
	b.rxBuilder = b.rxBuilder.CreatedAt(v)
	return b
}

func (b *Builder) CreatedByUser(v accountsid.UserID) *Builder {
	// TODO: after migration, remove this cast
	b.rxBuilder = b.rxBuilder.CreatedByUser(accountdomain.UserID(v))
	return b
}

func (b *Builder) CreatedByIntegration(v *id.IntegrationID) *Builder {
	b.integrationID = v
	if v != nil {
		rxID := ConvertIntegrationIDToReearthx(v)
		b.rxBuilder = b.rxBuilder.CreatedByIntegration(*rxID)
	}
	return b
}

func (b *Builder) FileName(v string) *Builder {
	b.rxBuilder = b.rxBuilder.FileName(v)
	return b
}

func (b *Builder) Name(v string) *Builder {
	b.rxBuilder = b.rxBuilder.Name(v)
	return b
}

func (b *Builder) Size(v uint64) *Builder {
	b.rxBuilder = b.rxBuilder.Size(v)
	return b
}

func (b *Builder) URL(v string) *Builder {
	b.rxBuilder = b.rxBuilder.URL(v)
	return b
}

func (b *Builder) ContentType(v string) *Builder {
	b.rxBuilder = b.rxBuilder.ContentType(v)
	return b
}

func (b *Builder) UUID(v string) *Builder {
	b.rxBuilder = b.rxBuilder.UUID(v)
	return b
}

func (b *Builder) NewUUID() *Builder {
	b.rxBuilder = b.rxBuilder.NewUUID()
	return b
}

func (b *Builder) Thread(v *id.ThreadID) *Builder {
	b.threadID = v
	if v != nil {
		b.rxBuilder = b.rxBuilder.Thread(ConvertThreadIDToReearthx(v))
	}
	return b
}

func (b *Builder) ArchiveExtractionStatus(v ArchiveExtractionStatus) *Builder {
	b.rxBuilder = b.rxBuilder.ArchiveExtractionStatus(&v)
	return b
}

func (b *Builder) FlatFiles(v bool) *Builder {
	b.rxBuilder = b.rxBuilder.FlatFiles(v)
	return b
}

func (b *Builder) Public(v bool) *Builder {
	b.rxBuilder = b.rxBuilder.Public(v)
	return b
}

func (b *Builder) Build() (*Asset, error) {
	rxAsset, err := b.rxBuilder.Build()
	if err != nil {
		return nil, err
	}
	return ConvertFromReearthx(rxAsset), nil
}

func (b *Builder) MustBuild() *Asset {
	rxAsset := b.rxBuilder.MustBuild()
	return ConvertFromReearthx(rxAsset)
}
