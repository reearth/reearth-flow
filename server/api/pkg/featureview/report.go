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

// Report is what a render leaves beside its output, and the server's only
// record of an outcome. It exists for every terminal outcome including the ones
// that wrote no view, which is what lets "this row holds 2D geometry" reach the
// user as an explanation instead of an exit code.
//
// Paths are relative to the view's own directory
// (`feature-view/{fileID}/`), not absolute URIs: the renderer does not know
// the public URL its output will be served from, and the server does not want
// to reverse a gs:// URI into one.
// Fields are ordered for struct packing (govet fieldalignment), not by
// meaning.
type Report struct {
	// Format is set only when Status is StatusReady.
	Format *Format `json:"format,omitempty"`
	Row    *int    `json:"row,omitempty"`
	Filter *string `json:"filter,omitempty"`
	// Error carries the renderer's message for a non-Ready outcome.
	Error *string `json:"error,omitempty"`
	// SizeLimitedFeatures counts selected features a vector-tile render left
	// out of at least one tile at its highest zoom, to keep that tile under its
	// size limit. A viewer zoomed in further still shows the highest-zoom
	// tiles, so where a feature was left out it is missing however far the
	// viewer zooms in. Features left out only at lower zooms, which are
	// overviews, are not counted. Nil for every other format: only vector tiles
	// have a size limit.
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
	// Layers are a ready view's entry points, one per format, 3D Tiles before
	// vector tiles; see ViewLayers. Format, EntryPoint and the size-limited
	// counts above repeat the first. Empty in a report from an engine that
	// wrote a single entry point.
	Layers  []ReportLayer `json:"layers,omitempty"`
	Version int           `json:"version"`
	// SelectedFeatures counts features the selection kept, before the writer
	// dropped any.
	SelectedFeatures int `json:"selectedFeatures"`
	// RenderedFeatures counts selected features drawn into the view, in any
	// of its layers: a feature drawn into two layers counts once.
	//
	// Fewer than SelectedFeatures is normal, not a warning. Geometry naming no
	// CRS is left out, and so is geometry the writer cannot draw, such as a
	// CSG: the feature keeps its row in the table and only its geometry is
	// absent from the view. In vector tiles a feature can also be left out of
	// every tile, either because it is smaller than a pixel at every zoom, or to
	// keep tiles under their size limit; SizeLimitedFeatures counts the second.
	// Together the counts are how a client explains a view that looks sparser
	// than the table.
	RenderedFeatures int `json:"renderedFeatures"`
	// Scanned counts non-empty lines the read examined. Short of the file's
	// total for a row selection, which stops at its row.
	Scanned int `json:"scanned"`
	// countsUnknown marks a report the renderer did not write; see
	// UnreadableReport.
	countsUnknown bool
}

// ReportLayer is one entry point of a ready view, and what was drawn into it.
// A tiles view of geometry holding both 3D and 2D has two: the 3D geometry in
// 3D Tiles and the 2D geometry in vector tiles.
//
// Fields are ordered for struct packing (govet fieldalignment), not by
// meaning.
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

// ViewLayers is a ready view's entry points. A report from an engine that
// wrote a single entry point names it at the top level, and that is the view's
// only layer. Nil for a view that is not ready, and for a ready report that
// names no format.
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

// UnreadableReport stands in for a report that exists but could not be parsed:
// the render happened, so it is answered as a failure rather than as "not
// rendered", which would send the caller into a render loop. It records no
// shape and no counts, because nothing about the render can be trusted.
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

// allowsFormat reports whether a shape can have produced a format. The engine
// chooses the format from the geometry it finds, but only within the shape it
// was asked for: a tiles render never yields a glb, and a gltf render never
// yields a tile pyramid. A report claiming otherwise did not come from a render
// the server asked for, and would hand the caller a tilejson URL for a glb.
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
	// Decode stops at the end of the first value, so trailing bytes would go
	// unseen. This parser is the validation boundary for the renderer's only
	// record of an outcome, and a report with anything after the object is not
	// one the renderer wrote.
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
