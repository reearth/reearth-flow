package gqlmodel

import (
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/featureview"
)

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
		ID:                  ID(r.Key),
		JobID:               ID(r.JobID.String()),
		FileID:              r.FileID,
		Shape:               ToIntermediateDataViewShape(r.Shape),
		Status:              ToIntermediateDataViewStatus(r.Status),
		Format:              ToIntermediateDataViewFormat(r.Format),
		SelectedFeatures:    r.SelectedFeatures,
		RenderedFeatures:    r.RenderedFeatures,
		SizeLimitedFeatures: r.SizeLimitedFeatures,
		SizeLimitedTiles:    r.SizeLimitedTiles,
		Error:               r.Error,
		// The list is non-null in the schema, so a view with no layers answers
		// an empty one.
		Layers: make([]*IntermediateDataViewLayer, 0, len(r.Layers)),
	}
	for _, layer := range r.Layers {
		if converted := ToIntermediateDataViewLayer(layer); converted != nil {
			view.Layers = append(view.Layers, converted)
		}
	}
	if r.EntryPointURL != "" {
		url := r.EntryPointURL
		view.EntryPointURL = &url
	}
	return view
}

// ToIntermediateDataViewLayer converts one entry point of a ready view, or
// returns nil for a format the schema does not name.
func ToIntermediateDataViewLayer(layer interfaces.IntermediateDataViewLayer) *IntermediateDataViewLayer {
	format := ToIntermediateDataViewFormat(&layer.Format)
	if format == nil {
		return nil
	}
	return &IntermediateDataViewLayer{
		Format:              *format,
		EntryPointURL:       layer.EntryPointURL,
		SelectedFeatures:    layer.SelectedFeatures,
		RenderedFeatures:    layer.RenderedFeatures,
		SizeLimitedFeatures: layer.SizeLimitedFeatures,
		SizeLimitedTiles:    layer.SizeLimitedTiles,
	}
}
