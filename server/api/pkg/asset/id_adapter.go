package asset

import (
	"time"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/account/accountdomain"
	reearthxasset "github.com/reearth/reearthx/asset/domain/asset"
	reearthxid "github.com/reearth/reearthx/asset/domain/id"
)

// ID type conversions between reearth-flow and reearthx

func ConvertAssetIDToReearthx(flowID id.AssetID) reearthxid.AssetID {
	rxID, _ := reearthxid.AssetIDFrom(flowID.String())
	return rxID
}

func ConvertAssetIDFromReearthx(rxID reearthxid.AssetID) id.AssetID {
	flowID, _ := id.AssetIDFrom(rxID.String())
	return flowID
}

func ConvertProjectIDToReearthx(flowID id.ProjectID) reearthxid.ProjectID {
	rxID, _ := reearthxid.ProjectIDFrom(flowID.String())
	return rxID
}

func ConvertProjectIDFromReearthx(rxID reearthxid.ProjectID) id.ProjectID {
	flowID, _ := id.ProjectIDFrom(rxID.String())
	return flowID
}

func ConvertThreadIDToReearthx(flowID *id.ThreadID) *reearthxid.ThreadID {
	if flowID == nil {
		return nil
	}
	tid, _ := reearthxid.ThreadIDFrom(flowID.String())
	return &tid
}

func ConvertThreadIDFromReearthx(rxID *reearthxid.ThreadID) *id.ThreadID {
	if rxID == nil {
		return nil
	}
	tid, _ := id.ThreadIDFrom(rxID.String())
	return &tid
}

func ConvertIntegrationIDToReearthx(flowID *id.IntegrationID) *reearthxid.IntegrationID {
	if flowID == nil {
		return nil
	}
	iid, _ := reearthxid.IntegrationIDFrom(flowID.String())
	return &iid
}

func ConvertIntegrationIDFromReearthx(rxID *reearthxid.IntegrationID) *id.IntegrationID {
	if rxID == nil {
		return nil
	}
	iid, _ := id.IntegrationIDFrom(rxID.String())
	return &iid
}

type AssetWrapper struct {
	asset *reearthxasset.Asset
}

func NewAssetWrapper(a *reearthxasset.Asset) *AssetWrapper {
	return &AssetWrapper{asset: a}
}

func (w *AssetWrapper) ReearthxAsset() *reearthxasset.Asset {
	return w.asset
}

func ConvertFromReearthx(rxAsset *reearthxasset.Asset) *AssetWrapper {
	if rxAsset == nil {
		return nil
	}
	return NewAssetWrapper(rxAsset)
}

func ConvertToReearthx(
	projectID id.ProjectID,
	workspaceID id.WorkspaceID,
	userID *id.UserID,
	integrationID *id.IntegrationID,
	fileName string,
	size uint64,
	url string,
	contentType string,
	threadID *id.ThreadID,
	archiveExtractionStatus *reearthxasset.ArchiveExtractionStatus,
	flatFiles bool,
	public bool,
) (*reearthxasset.Asset, error) {
	builder := reearthxasset.New().
		NewID().
		Workspace(accountdomain.WorkspaceID(workspaceID)). // TODO: after migration, remove this cast
		FileName(fileName).
		Name(fileName).
		Size(size).
		URL(url).
		ContentType(contentType).
		NewUUID().
		FlatFiles(flatFiles).
		Public(public)

	if userID != nil {
		builder = builder.CreatedByUser(accountdomain.UserID(*userID)) // TODO: after migration, remove this cast
	}

	if integrationID != nil {
		rxID := ConvertIntegrationIDToReearthx(integrationID)
		builder = builder.CreatedByIntegration(*rxID)
	}

	if threadID != nil {
		builder = builder.Thread(ConvertThreadIDToReearthx(threadID))
	}

	if archiveExtractionStatus != nil {
		builder = builder.ArchiveExtractionStatus(archiveExtractionStatus)
	}

	return builder.Build()
}

// Getter methods that handle ID conversions

func (w *AssetWrapper) ID() id.AssetID {
	return ConvertAssetIDFromReearthx(w.asset.ID())
}

func (w *AssetWrapper) Project() id.ProjectID {
	rxProject := w.asset.Project()
	flowProject := ConvertProjectIDFromReearthx(rxProject)

	// If this is our special workspace-only project ID, return empty
	if flowProject == WorkspaceOnlyProjectID {
		return id.ProjectID{}
	}

	return flowProject
}

func (w *AssetWrapper) Workspace() id.WorkspaceID {
	// TODO: after migration, remove this cast
	return id.WorkspaceID(w.asset.Workspace())
}

func (w *AssetWrapper) CreatedAt() time.Time {
	return w.asset.CreatedAt()
}

func (w *AssetWrapper) User() *id.UserID {
	a := w.asset.User()
	if a == nil {
		return nil
	}
	// TODO: after migration, remove this cast
	uid := id.UserID(*a)
	return &uid
}

func (w *AssetWrapper) Integration() *id.IntegrationID {
	return ConvertIntegrationIDFromReearthx(w.asset.Integration())
}

func (w *AssetWrapper) FileName() string {
	return w.asset.FileName()
}

func (w *AssetWrapper) Name() string {
	return w.asset.Name()
}

func (w *AssetWrapper) Size() uint64 {
	return w.asset.Size()
}

func (w *AssetWrapper) URL() string {
	return w.asset.URL()
}

func (w *AssetWrapper) ContentType() string {
	return w.asset.ContentType()
}

func (w *AssetWrapper) UUID() string {
	return w.asset.UUID()
}

func (w *AssetWrapper) Thread() *id.ThreadID {
	return ConvertThreadIDFromReearthx(w.asset.Thread())
}

func (w *AssetWrapper) ArchiveExtractionStatus() *reearthxasset.ArchiveExtractionStatus {
	return w.asset.ArchiveExtractionStatus()
}

func (w *AssetWrapper) FlatFiles() bool {
	return w.asset.FlatFiles()
}

func (w *AssetWrapper) Public() bool {
	return w.asset.Public()
}

// Utility functions to work with asset lists

func ConvertAssetListFromReearthx(rxAssets []*reearthxasset.Asset) []*AssetWrapper {
	wrappers := make([]*AssetWrapper, len(rxAssets))
	for i, a := range rxAssets {
		wrappers[i] = ConvertFromReearthx(a)
	}
	return wrappers
}
