package document

import "time"

type Document struct {
	id        ID
	updates   []int
	version   int
	timestamp time.Time
}

func NewDocument(id ID, updates []int, version int, timestamp time.Time) *Document {
	return &Document{
		id:        id,
		updates:   updates,
		version:   version,
		timestamp: timestamp,
	}
}

func (j *Document) ID() ID {
	return j.id
}

func (j *Document) Updates() []int {
	return j.updates
}

func (j *Document) Version() int {
	return j.version
}

func (j *Document) Timestamp() time.Time {
	return j.timestamp
}

func (j *Document) SetID(id ID) {
	j.id = id
}

func (j *Document) SetUpdates(updates []int) {
	j.updates = updates
}

func (j *Document) SetVersion(version int) {
	j.version = version
}

func (j *Document) SetTimestamp(timestamp time.Time) {
	j.timestamp = timestamp
}
