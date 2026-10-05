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
	Error  *string `json:"error,omitempty"`
	Status Status  `json:"status"`
	Shape  Shape   `json:"shape"`
	// EntryPoint is the file a viewer opens, relative to the view directory.
	// Empty unless Status is StatusReady.
	EntryPoint string `json:"entryPoint,omitempty"`
	// Written lists every file the render produced, the entry point included.
	Written []string `json:"written,omitempty"`
	Version int      `json:"version"`
	// SelectedFeatures counts features the selection kept, before the writer
	// dropped any.
	SelectedFeatures int `json:"selectedFeatures"`
	// RenderedFeatures counts selected features that reached the output.
	//
	// Fewer than SelectedFeatures is normal, not a warning: geometry naming no
	// CRS is dropped, and so is geometry the writer cannot draw, such as a CSG.
	// Dropping loses nothing — the feature keeps its row in the table and only
	// its geometry is absent from the view — so the two counts together are how
	// a client explains a view that looks sparser than the table.
	RenderedFeatures int `json:"renderedFeatures"`
	// Scanned counts non-empty lines the read examined. Short of the file's
	// total for a row selection, which stops at its row.
	Scanned int `json:"scanned"`
	// countsUnknown marks a report the renderer did not write; see
	// UnreadableReport.
	countsUnknown bool
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
	case StatusEmpty, StatusUnsupportedGeometry, StatusFailed:
		// These wrote no view, so there is nothing to point at.
	default:
		return nil, fmt.Errorf("%w: unknown status %q", ErrInvalidReport, report.Status)
	}

	return &report, nil
}
