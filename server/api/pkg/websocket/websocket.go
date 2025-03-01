package websocket

import "time"

type Document struct {
	ID        string
	Updates   []int
	Version   int
	Timestamp time.Time
}

type History struct {
	Updates   []int
	Version   int
	Timestamp time.Time
}
