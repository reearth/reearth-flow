package asset

import (
	reearthxasset "github.com/reearth/reearthx/asset/domain/asset"
)

// These are added to support reearth-flow's specific needs

const (
	PreviewTypeUnknownGeo PreviewType = "unknown_geo"
	PreviewTypeGeoJSON    PreviewType = "geojson"
	PreviewTypeGeoTIFF    PreviewType = "geotiff"
	PreviewTypeGPX        PreviewType = "gpx"
	PreviewTypeKML        PreviewType = "kml"
	PreviewTypeSHP        PreviewType = "shapefile"
	PreviewTypeCZML       PreviewType = "czml"
	PreviewTypePDF        PreviewType = "pdf"
	PreviewTypeHTML       PreviewType = "html"
	PreviewTypeXML        PreviewType = "xml"
	PreviewTypeText       PreviewType = "text"
	PreviewTypeJSON       PreviewType = "json"
	PreviewTypeSheet      PreviewType = "sheet"
	PreviewTypeArchive    PreviewType = "archive"
	PreviewTypeGLTF       PreviewType = "gltf"
	PreviewTypeVideo      PreviewType = "video"
	PreviewTypeAudio      PreviewType = "audio"
	PreviewTypeTMS        PreviewType = "tms"
	PreviewTypeGPKG       PreviewType = "gpkg"
)

func ExtendedPreviewTypeFrom(p string) (PreviewType, bool) {
	if pt, ok := reearthxasset.PreviewTypeFrom(p); ok {
		return pt, true
	}

	switch PreviewType(p) {
	case PreviewTypeUnknownGeo,
		PreviewTypeGeoJSON,
		PreviewTypeGeoTIFF,
		PreviewTypeGPX,
		PreviewTypeKML,
		PreviewTypeSHP,
		PreviewTypeCZML,
		PreviewTypePDF,
		PreviewTypeHTML,
		PreviewTypeXML,
		PreviewTypeText,
		PreviewTypeJSON,
		PreviewTypeSheet,
		PreviewTypeArchive,
		PreviewTypeGLTF,
		PreviewTypeVideo,
		PreviewTypeAudio,
		PreviewTypeTMS,
		PreviewTypeGPKG:
		return PreviewType(p), true
	}

	return PreviewTypeUnknown, false
}
