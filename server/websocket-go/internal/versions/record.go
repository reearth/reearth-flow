// Package versions saves a room's document as a version (a labelled snapshot,
// listed in the editor's Version panel) when users changed it, and records who
// did.
//
// Only edits users send count. Each instance reports the edits it receives to
// a change record kept in the coordination store, so every instance holding a
// room shares one record. A record names the items each user's edits inserted
// and deleted. Saving a version takes the record and deletes it in one step,
// then checks which of those edits the saved state holds: the rest go back
// into the record for the next version.
package versions

import (
	"context"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"sort"
	"sync"
	"time"

	"github.com/reearth/ygo/crdt"
)

// Edits is what one user's edits touched, and when they were made.
type Edits struct {
	// Inserts and Deletes are the IDs of the items the edits inserted and
	// deleted. Never nil.
	Inserts, Deletes *crdt.IDSet
	// FirstAt and LastAt are the times of the first and latest edit.
	FirstAt, LastAt time.Time
}

// NewEdits returns empty Edits.
func NewEdits() Edits {
	return Edits{Inserts: crdt.NewIDSet(), Deletes: crdt.NewIDSet()}
}

// add folds an update's content IDs, made at at, into e.
func (e *Edits) add(ids crdt.ContentIDs, at time.Time) {
	e.merge(Edits{Inserts: orEmpty(ids.Inserts), Deletes: orEmpty(ids.Deletes), FirstAt: at, LastAt: at})
}

func orEmpty(s *crdt.IDSet) *crdt.IDSet {
	if s == nil {
		return crdt.NewIDSet()
	}
	return s
}

// merge folds o into e: the IDs are united, the earliest first and the latest
// last edit kept.
func (e *Edits) merge(o Edits) {
	e.Inserts = crdt.MergeIDSets(e.Inserts, o.Inserts)
	e.Deletes = crdt.MergeIDSets(e.Deletes, o.Deletes)
	if !o.FirstAt.IsZero() && (e.FirstAt.IsZero() || o.FirstAt.Before(e.FirstAt)) {
		e.FirstAt = o.FirstAt
	}
	if o.LastAt.After(e.LastAt) {
		e.LastAt = o.LastAt
	}
}

func (e Edits) empty() bool { return e.Inserts.IsEmpty() && e.Deletes.IsEmpty() }

// Record is a room's change record: the user edits made since its last
// version, across every instance, by user. Edits from a connection that named
// no user are under "".
type Record map[string]Edits

// Merge folds o into r.
func (r Record) Merge(o Record) {
	for u, e := range o {
		cur, ok := r[u]
		if !ok {
			cur = NewEdits()
		}
		cur.merge(e)
		r[u] = cur
	}
}

// Contributors returns the users who edited, sorted.
func (r Record) Contributors() []string {
	out := make([]string, 0, len(r))
	for u := range r {
		if u != "" {
			out = append(out, u)
		}
	}
	sort.Strings(out)
	return out
}

// LastEditor returns the user who edited last and when; "" when no edit named
// its sender.
func (r Record) LastEditor() (string, time.Time) {
	var (
		who string
		at  time.Time
	)
	for u, e := range r {
		if u != "" && (who == "" || e.LastAt.After(at) || (e.LastAt.Equal(at) && u > who)) {
			who, at = u, e.LastAt
		}
	}
	return who, at
}

// FirstEditAt and LastEditAt bound every edit in r.
func (r Record) FirstEditAt() time.Time {
	var t time.Time
	for _, e := range r {
		if !e.FirstAt.IsZero() && (t.IsZero() || e.FirstAt.Before(t)) {
			t = e.FirstAt
		}
	}
	return t
}

func (r Record) LastEditAt() time.Time {
	var t time.Time
	for _, e := range r {
		if e.LastAt.After(t) {
			t = e.LastAt
		}
	}
	return t
}

// Split divides r into the edits the document encoded by state holds and the
// edits it does not. An insert is held when the document has the item; a
// delete is held when the document has the item deleted. A user whose edits
// are partly held appears in both.
func (r Record) Split(state []byte) (held, missing Record, err error) {
	d, err := decodeState(state)
	if err != nil {
		return nil, nil, err
	}
	held, missing = r.split(d)
	return held, missing, nil
}

// decodeState decodes a full state into a document.
func decodeState(state []byte) (*crdt.Doc, error) {
	d := crdt.New()
	if err := crdt.ApplyUpdateV1(d, state, nil); err != nil {
		return nil, fmt.Errorf("versions: decode state: %w", err)
	}
	return d, nil
}

// split is Split against an already decoded document.
func (r Record) split(d *crdt.Doc) (held, missing Record) {
	present := crdt.NewIDSet()
	for client, clock := range d.StateVector() {
		present.Add(client, 0, clock)
	}
	deleted := crdt.DeleteSetFromDoc(d)

	held, missing = Record{}, Record{}
	for u, e := range r {
		gone := Edits{
			Inserts: crdt.ExcludeIDSet(e.Inserts, present),
			Deletes: crdt.ExcludeIDSet(e.Deletes, deleted),
			FirstAt: e.FirstAt,
			LastAt:  e.LastAt,
		}
		if !gone.empty() {
			missing[u] = gone
		}
		have := Edits{
			Inserts: crdt.ExcludeIDSet(e.Inserts, gone.Inserts),
			Deletes: crdt.ExcludeIDSet(e.Deletes, gone.Deletes),
			FirstAt: e.FirstAt,
			LastAt:  e.LastAt,
		}
		// A batch can carry no IDs (an update the room could not decode for
		// IDs): it counts as held, so it is credited rather than kept forever.
		if !have.empty() || gone.empty() {
			held[u] = have
		}
	}
	return held, missing
}

// wireEdits is the stored form of one user's Edits.
type wireEdits struct {
	Inserts string `json:"i,omitempty"`
	Deletes string `json:"d,omitempty"`
	FirstAt int64  `json:"f"`
	LastAt  int64  `json:"l"`
}

// EncodeRecord encodes r for a Store that keeps batches as opaque values.
func EncodeRecord(r Record) ([]byte, error) {
	w := make(map[string]wireEdits, len(r))
	for u, e := range r {
		w[u] = wireEdits{
			Inserts: encodeIDSet(e.Inserts),
			Deletes: encodeIDSet(e.Deletes),
			FirstAt: unixMilli(e.FirstAt),
			LastAt:  unixMilli(e.LastAt),
		}
	}
	return json.Marshal(w)
}

// DecodeRecord decodes what EncodeRecord produced.
func DecodeRecord(b []byte) (Record, error) {
	var w map[string]wireEdits
	if err := json.Unmarshal(b, &w); err != nil {
		return nil, fmt.Errorf("versions: decode record: %w", err)
	}
	r := make(Record, len(w))
	for u, we := range w {
		ins, err := decodeIDSet(we.Inserts)
		if err != nil {
			return nil, err
		}
		del, err := decodeIDSet(we.Deletes)
		if err != nil {
			return nil, err
		}
		r[u] = Edits{Inserts: ins, Deletes: del, FirstAt: fromUnixMilli(we.FirstAt), LastAt: fromUnixMilli(we.LastAt)}
	}
	return r, nil
}

// unixMilli encodes t, and the zero time as 0, so a time that was never set
// decodes back as unset rather than as a date in 1754.
func unixMilli(t time.Time) int64 {
	if t.IsZero() {
		return 0
	}
	return t.UnixMilli()
}

func fromUnixMilli(ms int64) time.Time {
	if ms == 0 {
		return time.Time{}
	}
	return time.UnixMilli(ms)
}

// DecodeRecords decodes and merges stored batches.
func DecodeRecords(batches [][]byte) (Record, error) {
	r := Record{}
	for _, b := range batches {
		one, err := DecodeRecord(b)
		if err != nil {
			return nil, err
		}
		r.Merge(one)
	}
	return r, nil
}

func encodeIDSet(s *crdt.IDSet) string {
	if s == nil || s.IsEmpty() {
		return ""
	}
	return base64.StdEncoding.EncodeToString(crdt.EncodeIDSet(s))
}

func decodeIDSet(v string) (*crdt.IDSet, error) {
	if v == "" {
		return crdt.NewIDSet(), nil
	}
	b, err := base64.StdEncoding.DecodeString(v)
	if err != nil {
		return nil, fmt.Errorf("versions: decode id set: %w", err)
	}
	s, err := crdt.DecodeIDSet(b)
	if err != nil {
		return nil, fmt.Errorf("versions: decode id set: %w", err)
	}
	return s, nil
}

// Store keeps change records where every instance reaches them.
type Store interface {
	// Mark adds a batch of edits to room's record, creating the record if
	// there is none.
	Mark(ctx context.Context, room string, batch Record) error
	// Take returns room's record and deletes it in one atomic step, so each
	// batch goes to exactly one caller. ok is false when room has no record.
	Take(ctx context.Context, room string) (rec Record, ok bool, err error)
	// FirstEdit returns when room's oldest recorded edit was made; ok is false
	// when room has no record.
	FirstEdit(ctx context.Context, room string) (at time.Time, ok bool, err error)
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

func (s *MemoryStore) Mark(_ context.Context, room string, batch Record) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	r, ok := s.records[room]
	if !ok {
		r = Record{}
		s.records[room] = r
	}
	r.Merge(batch)
	return nil
}

func (s *MemoryStore) Take(_ context.Context, room string) (Record, bool, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	r, ok := s.records[room]
	delete(s.records, room)
	return r, ok, nil
}

func (s *MemoryStore) FirstEdit(_ context.Context, room string) (time.Time, bool, error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	r, ok := s.records[room]
	if !ok {
		return time.Time{}, false, nil
	}
	return r.FirstEditAt(), true, nil
}
