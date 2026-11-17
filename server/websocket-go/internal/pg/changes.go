package pg

import (
	"context"
	"errors"
	"sort"
	"time"

	"github.com/jackc/pgx/v5"

	"github.com/reearth/reearth-flow/websocket-go/internal/versions"
)

// markChangesSQL merges one batch into a room's change record, creating it.
// One statement, so concurrent marks from several instances serialise on the
// row. $3 is the batch's last editor, empty when none, and $4 when that edit was.
const markChangesSQL = `
INSERT INTO ws_version_change AS c (doc_id, contributors, last_editor, editor_at, first_at, last_at)
VALUES ($1, $2::text[], $3, CASE WHEN $3 = '' THEN NULL ELSE $4::timestamptz END, $5, $6)
ON CONFLICT (doc_id) DO UPDATE SET
  contributors = ARRAY(SELECT DISTINCT u FROM unnest(c.contributors || EXCLUDED.contributors) AS u ORDER BY u),
  last_editor = CASE
    WHEN EXCLUDED.last_editor <> '' AND (c.editor_at IS NULL OR EXCLUDED.editor_at >= c.editor_at)
    THEN EXCLUDED.last_editor ELSE c.last_editor END,
  editor_at = CASE
    WHEN EXCLUDED.last_editor <> '' AND (c.editor_at IS NULL OR EXCLUDED.editor_at >= c.editor_at)
    THEN EXCLUDED.editor_at ELSE c.editor_at END,
  first_at = LEAST(c.first_at, EXCLUDED.first_at),
  last_at = GREATEST(c.last_at, EXCLUDED.last_at)`

// takeChangesSQL reads a room's record and deletes it in one statement.
const takeChangesSQL = `
DELETE FROM ws_version_change WHERE doc_id = $1
RETURNING contributors, last_editor, editor_at, first_at, last_at`

const peekChangesSQL = `
SELECT contributors, last_editor, editor_at, first_at, last_at
FROM ws_version_change WHERE doc_id = $1`

// ChangeStore keeps change records in Postgres.
type ChangeStore struct {
	q Querier
}

// NewChangeStore returns a ChangeStore over q.
func NewChangeStore(q Querier) *ChangeStore {
	return &ChangeStore{q: q}
}

var _ versions.Store = (*ChangeStore)(nil)

func (s *ChangeStore) Mark(ctx context.Context, room string, m versions.Mark) error {
	editors := make([]string, 0, len(m.Editors))
	for _, u := range m.Editors {
		if u != "" {
			editors = append(editors, u)
		}
	}
	_, err := s.q.Exec(ctx, markChangesSQL, room, editors, m.LastEditor, m.LastEditorAt, m.FirstAt, m.LastAt)
	return err
}

func (s *ChangeStore) Take(ctx context.Context, room string) (versions.Record, bool, error) {
	return scanChanges(s.q.QueryRow(ctx, takeChangesSQL, room))
}

func (s *ChangeStore) Peek(ctx context.Context, room string) (versions.Record, bool, error) {
	return scanChanges(s.q.QueryRow(ctx, peekChangesSQL, room))
}

func scanChanges(row pgx.Row) (versions.Record, bool, error) {
	var (
		r        versions.Record
		editorAt *time.Time
	)
	err := row.Scan(&r.Contributors, &r.LastEditor, &editorAt, &r.FirstEditAt, &r.LastEditAt)
	if errors.Is(err, pgx.ErrNoRows) {
		return versions.Record{}, false, nil
	}
	if err != nil {
		return versions.Record{}, false, err
	}
	if editorAt != nil {
		r.LastEditorAt = *editorAt
	}
	if len(r.Contributors) == 0 {
		r.Contributors = nil
	}
	sort.Strings(r.Contributors)
	return r, true, nil
}
