package featureview

import (
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"strings"
)

// ReportVersion is the schema version the renderer writes and this package
// reads. Bump it on the renderer side only for a change readers cannot absorb.
const ReportVersion = 1

// Report is written beside every terminal render outcome, so a view that wrote
// nothing can still be explained to the user. Paths are relative to the view's
// directory (`feature-view/{fileID}/`) because the renderer does not know its
// public URL.
type Report struct {
	// Format is set only when Status is StatusReady.
	Format *Format `json:"format,omitempty"`
	Row    *int    `json:"row,omitempty"`
	Filter *string `json:"filter,omitempty"`
	// Error carries the renderer's message for a non-Ready outcome.
	Error *string `json:"error,omitempty"`
	// SizeLimitedFeatures counts features a vector-tile render left out of at
	// least one highest-zoom tile to stay under the size limit, which no further
	// zoom recovers. Nil for formats other than vector tiles.
	SizeLimitedFeatures *int `json:"sizeLimitedFeatures,omitempty"`
	// SizeLimitedTiles counts the tiles, at any zoom, that left out at least
	// one feature to stay under the size limit. Nil whenever
	// SizeLimitedFeatures is.
	SizeLimitedTiles *int   `json:"sizeLimitedTiles,omitempty"`
	Status           Status `json:"status"`
	Shape            Shape  `json:"shape"`
	// EntryPoint is the file a viewer opens, relative to the view directory.
	// Empty unless Status is StatusReady.
	EntryPoint string `json:"entryPoint,omitempty"`
	// EngineVersion is the engine that wrote the report. Empty for a report
	// written before engines named themselves.
	EngineVersion string `json:"engineVersion,omitempty"`
	// Written lists every file the render produced, the entry points included.
	Written []string `json:"written,omitempty"`
	// Layers are a ready view's entry points, one per format with 3D Tiles
	// first, and the top-level fields above repeat the first. Empty from engines
	// that wrote a single entry point; see ViewLayers.
	Layers  []ReportLayer `json:"layers,omitempty"`
	Version int           `json:"version"`
	// SelectedFeatures counts features the selection kept, before the writer
	// dropped any.
	SelectedFeatures int `json:"selectedFeatures"`
	// RenderedFeatures counts selected features drawn into any layer, once each.
	// It is normally below SelectedFeatures, because geometry with no CRS,
	// undrawable geometry, sub-pixel features and size-limited features are left
	// out of the view.
	RenderedFeatures int `json:"renderedFeatures"`
	// Scanned counts non-empty lines the read examined. Short of the file's
	// total for a row selection, which stops at its row.
	Scanned int `json:"scanned"`
	// countsUnknown marks a report the renderer did not write; see
	// UnreadableReport.
	countsUnknown bool
}

// ReportLayer is one entry point of a ready view, and what was drawn into it.
// A tiles view of mixed geometry has two: 3D Tiles and vector tiles.
type ReportLayer struct {
	// SizeLimitedFeatures and SizeLimitedTiles are what the layer's size limit
	// left out, as on Report. Nil for every format but vector tiles.
	SizeLimitedFeatures *int   `json:"sizeLimitedFeatures,omitempty"`
	SizeLimitedTiles    *int   `json:"sizeLimitedTiles,omitempty"`
	Format              Format `json:"format"`
	// EntryPoint is the file a viewer opens for this layer, relative to the
	// view directory.
	EntryPoint string `json:"entryPoint"`
	// SelectedFeatures counts the features this layer was given: the whole
	// selection, unless the view has two layers, when it is the features with
	// parts of this layer's dimension.
	SelectedFeatures int `json:"selectedFeatures"`
	// RenderedFeatures counts those of them drawn into this layer.
	RenderedFeatures int `json:"renderedFeatures"`
}

// ViewLayers is a ready view's entry points, falling back to the top-level one
// for engines that wrote a single entry point. Nil when the view is not ready
// or names no format.
func (r *Report) ViewLayers() []ReportLayer {
	if r.Status != StatusReady {
		return nil
	}
	if len(r.Layers) > 0 {
		return r.Layers
	}
	if r.Format == nil {
		return nil
	}
	return []ReportLayer{{
		Format:              *r.Format,
		EntryPoint:          r.EntryPoint,
		SelectedFeatures:    r.SelectedFeatures,
		RenderedFeatures:    r.RenderedFeatures,
		SizeLimitedFeatures: r.SizeLimitedFeatures,
		SizeLimitedTiles:    r.SizeLimitedTiles,
	}}
}

// UnreadableReport stands in for a report that exists but cannot be parsed. It
// is a failure rather than "not rendered", which would send the caller into a
// render loop.
func UnreadableReport(message string) *Report {
	return &Report{
		Version:       ReportVersion,
		Status:        StatusFailed,
		Error:         &message,
		countsUnknown: true,
	}
}

// CountsKnown reports whether SelectedFeatures and RenderedFeatures were
// recorded by a render. They are zero, not measured, on an UnreadableReport.
func (r *Report) CountsKnown() bool {
	return !r.countsUnknown
}

// allowsFormat reports whether a shape can have produced a format. A report
// claiming otherwise did not come from a render the server asked for.
func (s Shape) allowsFormat(f Format) error {
	switch s {
	case ShapeGLTF:
		if f == FormatGLB {
			return nil
		}
	case ShapeTiles:
		if f == FormatCesium3DTiles || f == FormatVectorTiles {
			return nil
		}
	default:
		return fmt.Errorf("%w: unknown shape %q", ErrInvalidReport, s)
	}
	return fmt.Errorf("%w: a %s report cannot have format %q", ErrInvalidReport, s, f)
}

// ParseReport reads a report and rejects one that contradicts itself, so a
// malformed or truncated object cannot be served to a client as a usable view.
func ParseReport(r io.Reader) (*Report, error) {
	var report Report
	dec := json.NewDecoder(r)
	if err := dec.Decode(&report); err != nil {
		return nil, fmt.Errorf("%w: %v", ErrInvalidReport, err)
	}
	// Decode stops after the first value, so trailing data is checked for
	// explicitly.
	if err := dec.Decode(new(json.RawMessage)); !errors.Is(err, io.EOF) {
		return nil, fmt.Errorf("%w: unexpected data after the report object", ErrInvalidReport)
	}

	if report.Version != ReportVersion {
		return nil, fmt.Errorf("%w: version %d is not the expected %d",
			ErrInvalidReport, report.Version, ReportVersion)
	}

	switch report.Status {
	case StatusReady:
		if report.Format == nil {
			return nil, fmt.Errorf("%w: a %s report must name its format", ErrInvalidReport, StatusReady)
		}
		if strings.TrimSpace(report.EntryPoint) == "" {
			return nil, fmt.Errorf("%w: a %s report must name its entry point", ErrInvalidReport, StatusReady)
		}
		if err := report.Shape.allowsFormat(*report.Format); err != nil {
			return nil, err
		}
		if err := report.validateLayers(); err != nil {
			return nil, err
		}
	case StatusEmpty, StatusUnsupportedGeometry, StatusFailed:
		// These wrote no view, so there is nothing to point at.
	default:
		return nil, fmt.Errorf("%w: unknown status %q", ErrInvalidReport, report.Status)
	}

	return &report, nil
}

// validateLayers rejects layers that contradict the report or each other. The
// top level repeats the first layer, a shape allows each layer's format, and a
// view has at most one layer per format.
func (r *Report) validateLayers() error {
	if len(r.Layers) == 0 {
		return nil
	}
	if r.Layers[0].Format != *r.Format || r.Layers[0].EntryPoint != r.EntryPoint {
		return fmt.Errorf("%w: the top level does not repeat the first layer", ErrInvalidReport)
	}
	seen := make(map[Format]bool, len(r.Layers))
	for _, layer := range r.Layers {
		if err := r.Shape.allowsFormat(layer.Format); err != nil {
			return err
		}
		if strings.TrimSpace(layer.EntryPoint) == "" {
			return fmt.Errorf("%w: a %s layer must name its entry point", ErrInvalidReport, layer.Format)
		}
		if seen[layer.Format] {
			return fmt.Errorf("%w: more than one %s layer", ErrInvalidReport, layer.Format)
		}
		seen[layer.Format] = true
	}
	return nil
}
