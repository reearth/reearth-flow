package gcs

import (
	"context"
	"unicode/utf8"
)

// ListAllDocs enumerates every persisted document id in the bucket so the admin
// full-bucket cleanup can compact all docs, including non-resident ones. It stays
// prefix-scoped (Phase 1: the OID-index keyspace; Phase 2: top-level "{id}/"
// folder prefixes) and never does an unscoped recursive bucket walk.
func (a *Adapter) ListAllDocs(ctx context.Context) ([]string, error) {
	if a.phase2 {
		return a.listAllDocsPhase2(ctx)
	}
	return a.listAllDocsPhase1(ctx)
}

// listAllDocsPhase1 scans the OID-index keyspace (prefix hex([V1, KEYSPACE_OID]) =
// "0000") to recover doc ids. The doc keyspace ("0001") shares the leading "00"
// byte, so names ≥ "0001" are filtered out; each surviving name decodes to key
// bytes whose doc id is key[2 : len-1] (drop header + terminator).
func (a *Adapter) listAllDocsPhase1(ctx context.Context) ([]string, error) {
	oidPrefix := hexb([]byte{rsV1, rsKeyspaceOID}) // "0000"
	docPrefix := hexb([]byte{rsV1, rsKeyspaceDoc}) // "0001"

	names, err := a.store.list(ctx, oidPrefix)
	if err != nil {
		return nil, err
	}

	seen := make(map[string]struct{}, len(names))
	out := make([]string, 0, len(names))
	for _, name := range names {
		// Defensive guard against doc-keyspace names (unreachable under the "0000"
		// scan prefix, but kept in case the prefix is ever widened).
		if name >= docPrefix {
			continue
		}
		kb, err := hexDecode(name)
		if err != nil || len(kb) <= 3 {
			continue
		}
		if kb[0] != rsV1 || kb[1] != rsKeyspaceOID {
			continue
		}
		idBytes := kb[2 : len(kb)-1] // drop header + TERMINATOR
		if !utf8.Valid(idBytes) {
			continue
		}
		id := string(idBytes)
		if _, dup := seen[id]; dup {
			continue
		}
		seen[id] = struct{}{}
		out = append(out, id)
	}
	return out, nil
}

// listAllDocsPhase2 lists the top-level "{id}/" folder prefixes (delimiter "/")
// and strips the trailing slash; the folder name is the doc id.
func (a *Adapter) listAllDocsPhase2(ctx context.Context) ([]string, error) {
	prefixes, err := a.store.listPrefixes(ctx, "")
	if err != nil {
		return nil, err
	}
	out := make([]string, 0, len(prefixes))
	for _, p := range prefixes {
		id := p
		if n := len(id); n > 0 && id[n-1] == '/' {
			id = id[:n-1]
		}
		if id == "" {
			continue
		}
		out = append(out, id)
	}
	return out, nil
}
