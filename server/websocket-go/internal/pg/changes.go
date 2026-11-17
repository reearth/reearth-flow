package pg

import (
	"context"
	"time"

	"github.com/reearth/reearth-flow/websocket-go/internal/versions"
)

// markChangesSQL adds one batch to a room's change record.
const markChangesSQL = `INSERT INTO ws_version_change (doc_id, first_at, batch) VALUES ($1, $2, $3)`

// takeChangesSQL reads a room's batches and deletes them in one statement. A
// batch inserted concurrently either commits before the delete's snapshot and
// is taken, or after it and stays for the next version.
const takeChangesSQL = `DELETE FROM ws_version_change WHERE doc_id = $1 RETURNING batch`

const firstChangeSQL = `SELECT min(first_at) FROM ws_version_change WHERE doc_id = $1`

// ChangeStore keeps change records in Postgres.
type ChangeStore struct {
	q Querier
}

// NewChangeStore returns a ChangeStore over q.
func NewChangeStore(q Querier) *ChangeStore {
	return &ChangeStore{q: q}
}

var _ versions.Store = (*ChangeStore)(nil)

func (s *ChangeStore) Mark(ctx context.Context, room string, batch versions.Record) error {
	value, err := versions.EncodeRecord(batch)
	if err != nil {
		return err
	}
	_, err = s.q.Exec(ctx, markChangesSQL, room, batch.FirstEditAt(), value)
	return err
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
