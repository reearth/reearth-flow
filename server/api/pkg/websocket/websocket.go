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

// SnapshotState is one snapshot's document state, keyed by the per-room
// SnapshotID, not the update-log version that Document and History carry.
type SnapshotState struct {
	Updates    []int
	SnapshotID int64
}
