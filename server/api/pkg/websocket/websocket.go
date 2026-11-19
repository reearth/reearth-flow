package websocket

import "time"

// Labels of the versions the API asks the websocket server to save. They are
// fixed codes, which the editor's version list shows as text of its own; the
// websocket server bounds the versions saved on request whatever their label.
const (
	// VersionLabelSave is a version saved from the editor (Ctrl+S).
	VersionLabelSave = "save"
	// VersionLabelRun is a version saved when a debug run starts.
	VersionLabelRun = "run"
	// VersionLabelDeploy is a version saved when the project is deployed.
	VersionLabelDeploy = "deploy"
)

type Document struct {
	Timestamp time.Time
	ID        string
	Updates   []int
	Version   int
}

type History struct {
	Timestamp time.Time
	Updates   []int
	Version   int
}

type HistoryMetadata struct {
	Timestamp time.Time
	Version   int
}

type SnapshotMetadata struct {
	Timestamp  time.Time
	Label      string
	LastEditor string
	// Contributors can be cut short, so ContributorCount can exceed its length.
	// Both are empty for snapshots saved without attribution.
	Contributors     []string
	ID               int64
	Size             int64
	ContributorCount int
}

// SnapshotState is one snapshot's document state, keyed by the per-room
// SnapshotID, not the update-log version that Document and History carry.
type SnapshotState struct {
	Updates    []int
	SnapshotID int64
}
