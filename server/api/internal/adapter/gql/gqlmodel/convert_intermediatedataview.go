package gqlmodel

import (
	"fmt"

	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/featureview"
)

// FromIntermediateDataViewShape maps the API's shape onto the domain's. The two
// vocabularies are kept separate so a schema rename cannot silently change what
// is sent to the renderer.
func FromIntermediateDataViewShape(shape IntermediateDataViewShape) (featureview.Shape, error) {
	switch shape {
	case IntermediateDataViewShapeGltf:
		return featureview.ShapeGLTF, nil
	case IntermediateDataViewShapeTiles:
		return featureview.ShapeTiles, nil
	default:
		return "", fmt.Errorf("unknown view shape %q", shape)
	}
}

func FromIntermediateDataViewTextureCodec(codec IntermediateDataViewTextureCodec) (featureview.TextureCodec, error) {
	switch codec {
	case IntermediateDataViewTextureCodecJpeg:
		return featureview.TextureCodecJPEG, nil
	case IntermediateDataViewTextureCodecPng:
		return featureview.TextureCodecPNG, nil
	case IntermediateDataViewTextureCodecKtx2Etc1s:
		return featureview.TextureCodecKTX2ETC1S, nil
	case IntermediateDataViewTextureCodecKtx2Uastc:
		return featureview.TextureCodecKTX2UASTC, nil
	case IntermediateDataViewTextureCodecUntextured:
		return featureview.TextureCodecUntextured, nil
	default:
		return "", fmt.Errorf("unknown texture codec %q", codec)
	}
}

// FromIntermediateDataViewOptions overlays the caller's options onto the
// defaults, so an omitted field keeps the server's default rather than becoming
// a zero that the domain would then reject.
//
// Ranges are checked by featureview so one rule covers every caller; only the
// int-to-sized-int narrowing is done here, because GraphQL Int is signed and
// the domain's counterparts are not.
func FromIntermediateDataViewOptions(in *IntermediateDataViewOptionsInput) (featureview.Options, error) {
	out := featureview.DefaultOptions()
	if in == nil {
		return out, nil
	}

	if in.Draco != nil {
		out.Draco = *in.Draco
	}
	if in.TexelSize != nil {
		out.TexelSize = *in.TexelSize
	}
	if in.TextureCodec != nil {
		codec, err := FromIntermediateDataViewTextureCodec(*in.TextureCodec)
		if err != nil {
			return out, err
		}
		out.TextureCodec = codec
	}
	if in.TargetTileSize != nil {
		v, err := nonNegative("targetTileSize", *in.TargetTileSize)
		if err != nil {
			return out, err
		}
		out.TargetTileSize = uint64(v)
	}
	if in.MinZoom != nil {
		v, err := zoom("minZoom", *in.MinZoom)
		if err != nil {
			return out, err
		}
		out.MinZoom = v
	}
	if in.MaxZoom != nil {
		v, err := zoom("maxZoom", *in.MaxZoom)
		if err != nil {
			return out, err
		}
		out.MaxZoom = v
	}
	if in.Extent != nil {
		out.Extent = int32(*in.Extent)
	}
	if in.MaxTileBytes != nil {
		v, err := nonNegative("maxTileBytes", *in.MaxTileBytes)
		if err != nil {
			return out, err
		}
		out.MaxTileBytes = uint64(v)
	}

	return out, nil
}

func nonNegative(field string, v int) (int, error) {
	if v < 0 {
		return 0, fmt.Errorf("%s must not be negative, got %d", field, v)
	}
	return v, nil
}

func zoom(field string, v int) (uint8, error) {
	if v < int(featureview.MinSupportedZoom) || v > int(featureview.MaxSupportedZoom) {
		return 0, fmt.Errorf("%s must be between %d and %d, got %d",
			field, featureview.MinSupportedZoom, featureview.MaxSupportedZoom, v)
	}
	return uint8(v), nil
}

func ToIntermediateDataViewShape(shape featureview.Shape) IntermediateDataViewShape {
	switch shape {
	case featureview.ShapeGLTF:
		return IntermediateDataViewShapeGltf
	default:
		return IntermediateDataViewShapeTiles
	}
}

func ToIntermediateDataViewStatus(status featureview.Status) IntermediateDataViewStatus {
	switch status {
	case featureview.StatusReady:
		return IntermediateDataViewStatusReady
	case featureview.StatusEmpty:
		return IntermediateDataViewStatusEmpty
	case featureview.StatusUnsupportedGeometry:
		return IntermediateDataViewStatusUnsupportedGeometry
	default:
		return IntermediateDataViewStatusFailed
	}
}

func ToIntermediateDataViewFormat(format *featureview.Format) *IntermediateDataViewFormat {
	if format == nil {
		return nil
	}
	var out IntermediateDataViewFormat
	switch *format {
	case featureview.FormatGLB:
		out = IntermediateDataViewFormatGlb
	case featureview.FormatCesium3DTiles:
		out = IntermediateDataViewFormatCesium3dTiles
	case featureview.FormatVectorTiles:
		out = IntermediateDataViewFormatVectorTiles
	default:
		return nil
	}
	return &out
}

func ToIntermediateDataView(r *interfaces.IntermediateDataViewResult) *IntermediateDataView {
	if r == nil {
		return nil
	}

	view := &IntermediateDataView{
		ID:               ID(r.Key),
		JobID:            ID(r.JobID.String()),
		FileID:           r.FileID,
		Shape:            ToIntermediateDataViewShape(r.Shape),
		Status:           ToIntermediateDataViewStatus(r.Status),
		Format:           ToIntermediateDataViewFormat(r.Format),
		SelectedFeatures: r.SelectedFeatures,
		RenderedFeatures: r.RenderedFeatures,
		Error:            r.Error,
	}
	if r.EntryPointURL != "" {
		url := r.EntryPointURL
		view.EntryPointURL = &url
	}
	return view
}
