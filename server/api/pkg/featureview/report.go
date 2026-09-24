package featureview

import (
	"encoding/json"
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
type Report struct {
	Version int    `json:"version"`
	Status  Status `json:"status"`
	Shape   Shape  `json:"shape"`
	// Format is set only when Status is StatusReady.
	Format *Format `json:"format,omitempty"`
	Row    *int    `json:"row,omitempty"`
	Filter *string `json:"filter,omitempty"`
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
	// EntryPoint is the file a viewer opens, relative to the view directory.
	// Empty unless Status is StatusReady.
	EntryPoint string `json:"entryPoint,omitempty"`
	// Written lists every file the render produced, the entry point included.
	Written []string `json:"written,omitempty"`
	// Error carries the renderer's message for a non-Ready outcome.
	Error *string `json:"error,omitempty"`
}

// ParseReport reads a report and rejects one that contradicts itself, so a
// malformed or truncated object cannot be served to a client as a usable view.
func ParseReport(r io.Reader) (*Report, error) {
	var report Report
	dec := json.NewDecoder(r)
	if err := dec.Decode(&report); err != nil {
		return nil, fmt.Errorf("%w: %v", ErrInvalidReport, err)
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
	case StatusEmpty, StatusUnsupportedGeometry, StatusFailed:
		// These wrote no view, so there is nothing to point at.
	default:
		return nil, fmt.Errorf("%w: unknown status %q", ErrInvalidReport, report.Status)
	}

	return &report, nil
}
