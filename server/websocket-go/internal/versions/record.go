// Package versions saves a room's document as a version (a labelled snapshot,
// listed in the editor's Version panel) when users changed it, and records who
// did.
//
// Only edits users send count. Each instance reports the edits it receives to
// a change record kept in the coordination store, so every instance holding a
// room shares one record. Saving a version takes the record and deletes it in
// one step: the instance that gets it saves the version, any other instance
// asking at the same moment finds no record and saves nothing.
package versions

import (
	"context"
	"slices"
	"sync"
	"time"
)

// Mark is a batch of user edits to one room, as one instance received them.
type Mark struct {
	// Editors are the users who sent the edits. Edits from a connection that
	// carries no user still count as changes.
	Editors []string
	// LastEditor sent the latest edit that named its sender, at LastEditorAt.
	LastEditor   string
	LastEditorAt time.Time
	// FirstAt and LastAt are the times of the batch's first and latest edit.
	FirstAt, LastAt time.Time
}

// Record is a room's change record: the user edits made since its last
// version, across every instance.
type Record struct {
	// Contributors are the users who edited, sorted.
	Contributors []string
	// LastEditor is the user who sent the latest edit, at LastEditorAt. Empty
	// when no edit named its sender.
	LastEditor   string
	LastEditorAt time.Time
	// FirstEditAt and LastEditAt bound the edits.
	FirstEditAt, LastEditAt time.Time
}

// Mark returns the record as a batch, so a record taken for a version that
// then failed can be merged back.
func (r Record) Mark() Mark {
	return Mark{
		Editors:      slices.Clone(r.Contributors),
		LastEditor:   r.LastEditor,
		LastEditorAt: r.LastEditorAt,
		FirstAt:      r.FirstEditAt,
		LastAt:       r.LastEditAt,
	}
}

// Merge folds m into r as Store.Mark does: contributors are united, the
// earliest first edit and the latest last edit kept, and the last editor is
// the one with the latest LastEditorAt.
func (r *Record) Merge(m Mark) {
	for _, u := range m.Editors {
		if u == "" {
			continue
		}
		if i, found := slices.BinarySearch(r.Contributors, u); !found {
			r.Contributors = slices.Insert(r.Contributors, i, u)
		}
	}
	if !m.FirstAt.IsZero() && (r.FirstEditAt.IsZero() || m.FirstAt.Before(r.FirstEditAt)) {
		r.FirstEditAt = m.FirstAt
	}
	if m.LastAt.After(r.LastEditAt) {
		r.LastEditAt = m.LastAt
	}
	if m.LastEditor != "" && (r.LastEditor == "" || !m.LastEditorAt.Before(r.LastEditorAt)) {
		r.LastEditor = m.LastEditor
		r.LastEditorAt = m.LastEditorAt
	}
}

// Store keeps change records where every instance reaches them.
type Store interface {
	// Mark merges m into room's record, creating the record if there is none.
	// See Record.Merge.
	Mark(ctx context.Context, room string, m Mark) error
	// Take returns room's record and deletes it in one atomic step, so each
	// record goes to exactly one caller. ok is false when room has no record.
	Take(ctx context.Context, room string) (rec Record, ok bool, err error)
	// Peek returns room's record and leaves it in place.
	Peek(ctx context.Context, room string) (rec Record, ok bool, err error)
}

// MemoryStore is a Store for a single instance.
type MemoryStore struct {
	mu      sync.Mutex
	records map[string]Record
}

// NewMemoryStore returns an empty MemoryStore.
func NewMemoryStore() *MemoryStore {
	return &MemoryStore{records: map[string]Record{}}
}

var _ Store = (*MemoryStore)(nil)

func (s *MemoryStore) Mark(_ context.Context, room string, m Mark) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	r := s.records[room]
	r.Merge(m)
	s.records[room] = r
	return nil
}

func (s *MemoryStore) Take(_ context.Context, room string) (Record, bool, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	r, ok := s.records[room]
	delete(s.records, room)
	return r, ok, nil
}

func (s *MemoryStore) Peek(_ context.Context, room string) (Record, bool, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	r, ok := s.records[room]
	r.Contributors = slices.Clone(r.Contributors)
	return r, ok, nil
}
