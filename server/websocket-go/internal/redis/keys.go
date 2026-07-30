// Package redis implements ygo's cluster.Relay over Redis Streams, byte- and
// semantics-compatible with the reearth-flow Rust WebSocket server so Rust and
// Go instances coexist on a single Redis during a blue-green rollout.
//
// SECURITY: the only client-derived value interpolated into a key is the
// already-validated doc id, and only as the trailing segment of a fixed prefix;
// every Lua script is parameterized via KEYS/ARGV. The stream clientId is a
// per-process random u64 used solely for self-filtering, never as an author or
// authz signal.
package redis

const (
	msgTypeSync      = "sync"
	msgTypeAwareness = "awareness"
)

// oidLockKey is the global OID-allocation lock, owned by the GCS layer; the relay
// never touches it.
const oidLockKey = "lock:oid_generation"

// streamKey is the per-doc stream carrying both sync and awareness updates.
func streamKey(docID string) string { return "yjs:stream:" + docID }

// instancesKey is the per-doc heartbeat hash: field=clientId, val=epoch-secs.
func instancesKey(docID string) string { return "doc:instances:" + docID }

// lockKey is the per-doc evict/save lock (TTL 10s).
func lockKey(docID string) string { return "lock:doc:" + docID }

// readLockKey protects batched stream reads from deletion (TTL 30s).
func readLockKey(docID string) string { return "read:lock:" + docID }

// gcsLockKey is the GCS-save lock. The doubled prefix reproduces a Rust quirk
// bug-for-bug — the real key is "lock:doc:gcs:lock:{doc}". Do NOT "fix" it during
// Phase 1 or two writers could both hold the GCS lock.
func gcsLockKey(docID string) string { return "lock:doc:gcs:lock:" + docID }
