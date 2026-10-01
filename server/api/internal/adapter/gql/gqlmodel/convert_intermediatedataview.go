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
