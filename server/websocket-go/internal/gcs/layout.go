// Package gcs implements ygo's persistence.VersionedPersistence against Google
// Cloud Storage, byte-compatible with the legacy ROOT layout (Phase 1) and the
// per-{projectId}/ layout (Phase 2). It owns the reearth-flow GCS byte layout:
// the single-hex v1 keyspace, the double-hex of doc_v2/checkpoint, OID mechanics,
// and brotli.
//
// In Phase 2 the doc id D is used RAW as a path prefix, so it MUST pass
// ValidateDocIDForPrefix first. Phase 1 hex-encodes every name, so traversal is
// neutralized there for free.
package gcs

import (
	"encoding/binary"
	"encoding/hex"
	"errors"
	"strings"
)

// DocID is the opaque, normalized document identifier (== projectId).
type DocID = string

// Layout produces final GCS object names.
type Layout interface {
	OIDIndexName(d DocID) string
	DocStateName(d DocID, oid uint32) string
	StateVectorName(d DocID, oid uint32) string
	UpdateName(d DocID, oid, clock uint32) string
	DocV2Name(d DocID) string
	CheckpointName(d DocID) string
	UpdatePrefix(d DocID, oid uint32) string // for List/PruneAfter/Compact
}

func hexb(b []byte) string { return hex.EncodeToString(b) }

func hexDecode(s string) ([]byte, error) { return hex.DecodeString(s) }

func be32(x uint32) []byte {
	b := make([]byte, 4)
	binary.BigEndian.PutUint32(b, x)
	return b
}

func be32ToU32(b []byte) uint32 { return binary.BigEndian.Uint32(b) }

const (
	rsV1          = 0x00
	rsKeyspaceOID = 0x00
	rsKeyspaceDoc = 0x01
	rsSubDoc      = 0x00
	rsSubStateVec = 0x01
	rsSubUpdate   = 0x02
	rsTerminator  = 0x00
)

// v1key builds a structured v1 keyspace key: V1 ‖ KEYSPACE_DOC ‖ oid(4 BE) ‖ sub ‖
// tail. The leading [V1, KEYSPACE_DOC]=[0,1] header is load-bearing: the update
// clock is read at key_bytes[7..11], which only lines up with it present.
func v1key(oid uint32, sub byte, tail []byte) []byte {
	k := make([]byte, 0, 7+len(tail))
	k = append(k, rsV1, rsKeyspaceDoc)
	k = append(k, be32(oid)...)
	k = append(k, sub)
	k = append(k, tail...)
	return k
}

// SystemLastOIDName is the global last-OID object: hex(utf8("system:last_oid")).
func SystemLastOIDName() string { return hexb([]byte("system:last_oid")) }

// LegacyRootLayout is the Phase-1 layout: v1 keyspace keys single-hex,
// doc_v2/checkpoint DOUBLE-hex.
type LegacyRootLayout struct{}

// OIDIndexName: V1 ‖ KEYSPACE_OID ‖ utf8(D) ‖ TERMINATOR, single-hex.
func (LegacyRootLayout) OIDIndexName(d DocID) string {
	k := make([]byte, 0, len(d)+3)
	k = append(k, rsV1, rsKeyspaceOID)
	k = append(k, []byte(d)...)
	k = append(k, rsTerminator)
	return hexb(k)
}

func (LegacyRootLayout) DocStateName(_ DocID, oid uint32) string {
	return hexb(v1key(oid, 0x00, nil))
}

func (LegacyRootLayout) StateVectorName(_ DocID, oid uint32) string {
	return hexb(v1key(oid, 0x01, nil))
}

func (LegacyRootLayout) UpdateName(_ DocID, oid, clock uint32) string {
	return hexb(v1key(oid, 0x02, append(be32(clock), 0x00)))
}

// DocV2Name is DOUBLE-hex: hex(utf8("doc_v2:" + hex(utf8(D)))).
func (LegacyRootLayout) DocV2Name(d DocID) string {
	return hexb([]byte("doc_v2:" + hexb([]byte(d))))
}

// CheckpointName is DOUBLE-hex: hex(utf8("checkpoint:" + hex(utf8(D)))).
func (LegacyRootLayout) CheckpointName(d DocID) string {
	return hexb([]byte("checkpoint:" + hexb([]byte(d))))
}

// UpdatePrefix is the hex prefix shared by all update@clock objects for an oid:
// hex(V1 ‖ KEYSPACE_DOC ‖ oid ‖ SUB_UPDATE).
func (LegacyRootLayout) UpdatePrefix(_ DocID, oid uint32) string {
	return hexb(updatePrefixBytes(oid))
}

func updatePrefixBytes(oid uint32) []byte {
	k := make([]byte, 0, 7)
	k = append(k, rsV1, rsKeyspaceDoc)
	k = append(k, be32(oid)...)
	k = append(k, rsSubUpdate)
	return k
}

// FolderOID is the constant per-folder OID (one doc per folder ⇒ no allocation).
const FolderOID uint32 = 0

// ProjectFolderLayout places every doc under "{D}/" with OID=0 and no double-hex.
// Callers MUST validate D via ValidateDocIDForPrefix before constructing names.
type ProjectFolderLayout struct{}

func (ProjectFolderLayout) OIDIndexName(_ DocID) string { return "" }

func (ProjectFolderLayout) DocStateName(d DocID, _ uint32) string {
	return string(d) + "/" + hexb(v1key(FolderOID, 0x00, nil))
}

func (ProjectFolderLayout) StateVectorName(d DocID, _ uint32) string {
	return string(d) + "/" + hexb(v1key(FolderOID, 0x01, nil))
}

func (ProjectFolderLayout) UpdateName(d DocID, _, clock uint32) string {
	return string(d) + "/" + hexb(v1key(FolderOID, 0x02, append(be32(clock), 0x00)))
}

func (ProjectFolderLayout) DocV2Name(d DocID) string      { return string(d) + "/doc_v2" }
func (ProjectFolderLayout) CheckpointName(d DocID) string { return string(d) + "/checkpoint" }

func (ProjectFolderLayout) UpdatePrefix(d DocID, _ uint32) string {
	return string(d) + "/" + hexb(updatePrefixBytes(FolderOID))
}

// ProjectPrefix is the {D}/ prefix scoping every list/delete to one project.
func ProjectPrefix(d DocID) string { return string(d) + "/" }

// ErrUnsafeDocID is returned by ValidateDocIDForPrefix for a doc id that cannot
// be safely used as a GCS path prefix.
var ErrUnsafeDocID = errors.New("gcs: doc id unsafe for use as a path prefix")

// ValidateDocIDForPrefix rejects a doc id that would traverse or forge object
// names when used raw as a Phase-2 "{D}/" path prefix: empty/whitespace, any '/',
// "." or ".." segments, control chars, leading/trailing whitespace. It does NOT
// parse ':' (opacity) and does NOT UUID-gate.
func ValidateDocIDForPrefix(d DocID) error {
	if d == "" || strings.TrimSpace(d) == "" {
		return ErrUnsafeDocID
	}
	if d != strings.TrimSpace(d) {
		return ErrUnsafeDocID
	}
	if strings.ContainsRune(d, '/') {
		return ErrUnsafeDocID
	}
	if d == "." || d == ".." {
		return ErrUnsafeDocID
	}
	for _, r := range d {
		if r < 0x20 || r == 0x7f {
			return ErrUnsafeDocID
		}
	}
	return nil
}
