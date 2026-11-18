package pg

import (
	"context"
	"errors"
	"hash/fnv"
	"sync"
	"time"

	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"

	"github.com/reearth/reearth-flow/websocket-go/internal/versions"
)

// readChangesSQL reads the edits one instance recorded for a room.
const readChangesSQL = `SELECT batch FROM ws_version_change WHERE doc_id = $1 AND instance = $2`

// insertChangesSQL starts an instance's row; it inserts nothing when the row
// appeared meanwhile.
const insertChangesSQL = `INSERT INTO ws_version_change (doc_id, instance, first_at, batch)
VALUES ($1, $2, $3, $4) ON CONFLICT (doc_id, instance) DO NOTHING`

// updateChangesSQL replaces an instance's row if it still holds what was read.
const updateChangesSQL = `UPDATE ws_version_change SET batch = $4, first_at = least(first_at, $3)
WHERE doc_id = $1 AND instance = $2 AND batch = $5`

// takeChangesSQL reads a room's rows and deletes them in one statement. A row
// written concurrently either commits before the delete's snapshot and is
// taken, or after it and stays for the next version.
const takeChangesSQL = `DELETE FROM ws_version_change WHERE doc_id = $1 RETURNING batch`

const firstChangeSQL = `SELECT min(first_at) FROM ws_version_change WHERE doc_id = $1`

// markAttempts bounds Mark's compare-and-set retries. Only this instance
// writes its row, so a retry follows a Take, and a second one is rare.
const markAttempts = 5

// errMarkContended reports that Mark kept finding its row changed.
var errMarkContended = errors.New("pg: change record kept changing while being written")

// ChangeStore keeps change records in Postgres, one row per room and instance.
type ChangeStore struct {
	q        Querier
	instance string
	// stripes serialize this instance's Marks of a room, so the
	// compare-and-set only ever retries after a Take.
	stripes [64]sync.Mutex
}

// NewChangeStore returns a ChangeStore over q for the instance named instance,
// which must be unique across the instances sharing the database.
func NewChangeStore(q Querier, instance string) *ChangeStore {
	return &ChangeStore{q: q, instance: instance}
}

var _ versions.Store = (*ChangeStore)(nil)

// Mark merges batch into this instance's row: it reads the row, merges in Go,
// and writes the result only if the row is unchanged, retrying otherwise.
func (s *ChangeStore) Mark(ctx context.Context, room string, batch versions.Record) error {
	first := batch.FirstEditAt()
	if first.IsZero() {
		first = time.Now()
	}
	mu := &s.stripes[stripeOf(room)]
	mu.Lock()
	defer mu.Unlock()
	for range markAttempts {
		var old []byte
		err := s.q.QueryRow(ctx, readChangesSQL, room, s.instance).Scan(&old)
		found := err == nil
		if err != nil && !errors.Is(err, pgx.ErrNoRows) {
			return err
		}
		merged := versions.Record{}
		if found {
			if merged, err = versions.DecodeRecord(old); err != nil {
				return err
			}
		}
		merged.Merge(batch)
		value, err := versions.EncodeRecord(merged)
		if err != nil {
			return err
		}
		var tag pgconn.CommandTag
		if found {
			tag, err = s.q.Exec(ctx, updateChangesSQL, room, s.instance, first, value, old)
		} else {
			tag, err = s.q.Exec(ctx, insertChangesSQL, room, s.instance, first, value)
		}
		if err != nil {
			return err
		}
		if tag.RowsAffected() == 1 {
			return nil
		}
	}
	return errMarkContended
}

func (s *ChangeStore) Take(ctx context.Context, room string) (versions.Record, bool, error) {
	rows, err := s.q.Query(ctx, takeChangesSQL, room)
	if err != nil {
		return nil, false, err
	}
	defer rows.Close()
	var batches [][]byte
	for rows.Next() {
		var b []byte
		if err := rows.Scan(&b); err != nil {
			return nil, false, err
		}
		batches = append(batches, b)
	}
	if err := rows.Err(); err != nil {
		return nil, false, err
	}
	if len(batches) == 0 {
		return nil, false, nil
	}
	rec, err := versions.DecodeRecords(batches)
	if err != nil {
		return nil, false, err
	}
	return rec, true, nil
}

func (s *ChangeStore) FirstEdit(ctx context.Context, room string) (time.Time, bool, error) {
	var at *time.Time
	if err := s.q.QueryRow(ctx, firstChangeSQL, room).Scan(&at); err != nil {
		return time.Time{}, false, err
	}
	if at == nil {
		return time.Time{}, false, nil
	}
	return *at, true, nil
}

func stripeOf(room string) int {
	h := fnv.New32a()
	_, _ = h.Write([]byte(room))
	return int(h.Sum32() % 64)
}
