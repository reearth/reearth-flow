package asset

import (
	"time"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/account/accountdomain"
	reearthxasset "github.com/reearth/reearthx/asset/domain/asset"
)

// WorkspaceOnlyProjectID is a special project ID used for workspace-only assets
// This is needed because reearthx requires a valid project ID
var WorkspaceOnlyProjectID = func() id.ProjectID {
	// Create a new project ID that we'll use as a marker for workspace-only assets
	return id.NewProjectID()
}()

// Re-export types from reearthx
type (
	Asset                   = AssetWrapper // Use wrapper as the main Asset type
	ArchiveExtractionStatus = reearthxasset.ArchiveExtractionStatus
	SortType                = reearthxasset.SortType
)

// Re-export constants from reearthx
const (
	ArchiveExtractionStatusPending    = reearthxasset.ArchiveExtractionStatusPending
	ArchiveExtractionStatusInProgress = reearthxasset.ArchiveExtractionStatusInProgress
	ArchiveExtractionStatusDone       = reearthxasset.ArchiveExtractionStatusDone
	ArchiveExtractionStatusFailed     = reearthxasset.ArchiveExtractionStatusFailed
	ArchiveExtractionStatusSkipped    = reearthxasset.ArchiveExtractionStatusSkipped
)

// Re-export sort types from reearthx
var (
	SortTypeID   = reearthxasset.SortTypeID
	SortTypeNAME = reearthxasset.SortTypeName
	SortTypeSIZE = reearthxasset.SortTypeSize
	SortTypeDATE = SortType{Key: "date"} // Not available in reearthx, add it
)

// Re-export functions from reearthx
var (
	ArchiveExtractionStatusFrom = reearthxasset.ArchiveExtractionStatusFrom
)

// ID generation functions - these use flow's ID types
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

func (b *Builder) Workspace(v id.WorkspaceID) *Builder {
	// TODO: after migration, remove this cast
	b.rxBuilder = b.rxBuilder.Workspace(accountdomain.WorkspaceID(v))
	// If no project is set yet, use a dummy project ID for workspace-based assets
	// This is required because reearthx requires a valid project ID
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

func (b *Builder) CreatedByUser(v id.UserID) *Builder {
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
