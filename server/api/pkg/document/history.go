package document

import "time"

type History struct {
	updates   []int
	version   int
	timestamp time.Time
}

func NewHistory(updates []int, version int, timestamp time.Time) *History {
	return &History{
		updates:   updates,
		version:   version,
		timestamp: timestamp,
	}
}

func (j *History) Updates() []int {
	return j.updates
}

func (j *History) Version() int {
	return j.version
}

func (j *History) Timestamp() time.Time {
	return j.timestamp
}

func (j *History) SetUpdates(updates []int) {
	j.updates = updates
}

func (j *History) SetVersion(version int) {
	j.version = version
}

func (j *History) SetTimestamp(timestamp time.Time) {
	j.timestamp = timestamp
}
