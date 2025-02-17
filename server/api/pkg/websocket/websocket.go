package websocket

import "time"

type Document struct {
	ID        string
	Update    []int
	Clock     int
	Timestamp time.Time
}

type History struct {
	Update    []int
	Clock     int
	Timestamp time.Time
}
