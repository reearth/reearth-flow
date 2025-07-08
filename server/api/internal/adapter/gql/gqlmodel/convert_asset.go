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
		ProjectID:               IDFrom(a.Project()),
		WorkspaceID:             IDFrom(a.Workspace()),
		CreatedAt:               a.CreatedAt(),
		FileName:                a.FileName(),
		Size:                    int64(a.Size()),
		ContentType:             a.ContentType(),
		Name:                    a.Name(),
		URL:                     a.URL(),
		UUID:                    a.UUID(),
		PreviewType:             ToPreviewType(a.PreviewType()),
		CoreSupport:             a.CoreSupport(),
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

func ToPreviewType(pt *asset.PreviewType) *PreviewType {
	if pt == nil {
		return nil
	}

	var result PreviewType
	switch *pt {
	case asset.PreviewTypeImage:
		result = PreviewTypeImage
	case asset.PreviewTypeImageSvg:
		result = PreviewTypeImageSVG
	case asset.PreviewTypeGeo:
		result = PreviewTypeGeo
	case asset.PreviewTypeGeo3dTiles:
		result = PreviewTypeGeo3dTiles
	case asset.PreviewTypeGeoMvt:
		result = PreviewTypeGeoMvt
	case asset.PreviewTypeModel3d:
		result = PreviewTypeModel3d
	case asset.PreviewTypeCSV:
		result = PreviewTypeCSV
	case asset.PreviewTypeUnknown:
		result = PreviewTypeUnknown
	default:
		// Handle extended preview types using string comparison
		switch string(*pt) {
		case "unknown_geo":
			result = PreviewTypeUnknownGeo
		case "geojson":
			result = PreviewTypeGeojson
		case "geotiff":
			result = PreviewTypeGeotiff
		case "gpx":
			result = PreviewTypeGpx
		case "kml":
			result = PreviewTypeKml
		case "shapefile":
			result = PreviewTypeShp
		case "czml":
			result = PreviewTypeCzml
		case "pdf":
			result = PreviewTypePDF
		case "html":
			result = PreviewTypeHTML
		case "xml":
			result = PreviewTypeXML
		case "text":
			result = PreviewTypeText
		case "json":
			result = PreviewTypeJSON
		case "sheet":
			result = PreviewTypeSheet
		case "archive":
			result = PreviewTypeArchive
		case "gltf":
			result = PreviewTypeGltf
		case "video":
			result = PreviewTypeVideo
		case "audio":
			result = PreviewTypeAudio
		case "tms":
			result = PreviewTypeTms
		case "gpkg":
			result = PreviewTypeGpkg
		default:
			result = PreviewTypeUnknown
		}
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
