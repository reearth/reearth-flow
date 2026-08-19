package websocket

import "time"

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
	Timestamp time.Time
	Label     string
	ID        int64
	Size      int64
}

// SnapshotState is one snapshot's stored document state.
//
// Addressed by SnapshotID, the per-room snapshot counter, NOT the update-log
// version that Document and History carry. The two are unrelated backend-assigned
// id spaces; passing one where the other is expected reads or prunes an unrelated
// point in history.
type SnapshotState struct {
	Updates    []int
	SnapshotID int64
}
