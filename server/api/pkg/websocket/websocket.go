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
