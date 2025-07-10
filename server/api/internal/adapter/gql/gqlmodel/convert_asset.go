package gqlmodel

import (
	"github.com/reearth/reearth-flow/api/pkg/asset"
)

func ToAsset(a *asset.Asset) *Asset {
	if a == nil {
		return nil
	}

	return &Asset{
		ID:                      IDFrom(a.ID()),
		WorkspaceID:             IDFrom(a.Workspace()),
		CreatedAt:               a.CreatedAt(),
		FileName:                a.FileName(),
		Size:                    int64(a.Size()),
		ContentType:             a.ContentType(),
		Name:                    a.Name(),
		URL:                     a.URL(),
		UUID:                    a.UUID(),
		FlatFiles:               a.FlatFiles(),
		Public:                  a.Public(),
		ArchiveExtractionStatus: ToArchiveExtractionStatus(a.ArchiveExtractionStatus()),
	}
}

func AssetSortTypeFrom(ast *AssetSortType) *asset.SortType {
	if ast == nil {
		return nil
	}

	var result asset.SortType
	switch *ast {
	case AssetSortTypeDate:
		result = asset.SortTypeID
	case AssetSortTypeName:
		result = asset.SortTypeNAME
	case AssetSortTypeSize:
		result = asset.SortTypeSIZE
	default:
		result = asset.SortTypeID
	}
	return &result
}

func ToArchiveExtractionStatus(s *asset.ArchiveExtractionStatus) *ArchiveExtractionStatus {
	if s == nil {
		return nil
	}

	var result ArchiveExtractionStatus
	switch *s {
	case asset.ArchiveExtractionStatusSkipped:
		result = ArchiveExtractionStatusSkipped
	case asset.ArchiveExtractionStatusPending:
		result = ArchiveExtractionStatusPending
	case asset.ArchiveExtractionStatusInProgress:
		result = ArchiveExtractionStatusInProgress
	case asset.ArchiveExtractionStatusDone:
		result = ArchiveExtractionStatusDone
	case asset.ArchiveExtractionStatusFailed:
		result = ArchiveExtractionStatusFailed
	default:
		result = ArchiveExtractionStatusSkipped
	}
	return &result
}
