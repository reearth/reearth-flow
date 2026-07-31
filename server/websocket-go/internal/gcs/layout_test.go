package gcs

import (
	"strings"
	"testing"
)

// Golden object-name vectors. Getting the double-hex of doc_v2/checkpoint wrong
// silently writes to a valid-looking but wrong object.

func TestLegacyRootLayout_GoldenNames(t *testing.T) {
	l := LegacyRootLayout{}
	const oid = uint32(7)

	tests := []struct {
		name string
		got  string
		want string
	}{
		{"oid-index proj1", l.OIDIndexName("proj1"), "000070726f6a3100"},
		{"doc_v2 proj1 (double-hex)", l.DocV2Name("proj1"), "646f635f76323a37303732366636613331"},
		{"checkpoint proj1 (double-hex)", l.CheckpointName("proj1"), "636865636b706f696e743a37303732366636613331"},
		{"system:last_oid", SystemLastOIDName(), "73797374656d3a6c6173745f6f6964"},
	}
	for _, tt := range tests {
		if tt.got != tt.want {
			t.Errorf("%s = %q, want %q", tt.name, tt.got, tt.want)
		}
	}
	_ = oid
}

// Full UUID double-hex doc_v2 object name.
func TestLegacyRootLayout_UUIDDoubleHex(t *testing.T) {
	l := LegacyRootLayout{}
	const uuid = "01234567-89ab-cdef-0123-456789abcdef"
	const wantDocV2 = "646f635f76323a333033313332333333343335333633373264333833393631363232643633363436353636326433303331333233333264333433353336333733383339363136323633363436353636"
	if got := l.DocV2Name(uuid); got != wantDocV2 {
		t.Errorf("DocV2Name(uuid) =\n  %q\nwant\n  %q", got, wantDocV2)
	}
}

// v1 structured keyspace: single-hex of V1 ‖ KEYSPACE_DOC ‖ OID(4 BE) ‖ sub ‖ tail.
func TestLegacyRootLayout_V1Keyspace(t *testing.T) {
	l := LegacyRootLayout{}
	const d = "proj1"
	const hdr = "0001" + "00000007" // V1 KEYSPACE_DOC oid=7
	if got, want := l.DocStateName(d, 7), hdr+"00"; got != want {
		t.Errorf("DocStateName = %q, want %q", got, want)
	}
	if got, want := l.StateVectorName(d, 7), hdr+"01"; got != want {
		t.Errorf("StateVectorName = %q, want %q", got, want)
	}
	if got, want := l.UpdateName(d, 7, 5), hdr+"02"+"00000005"+"00"; got != want {
		t.Errorf("UpdateName = %q, want %q", got, want)
	}
	if got, want := l.UpdatePrefix(d, 7), hdr+"02"; got != want {
		t.Errorf("UpdatePrefix = %q, want %q", got, want)
	}
}

// Phase-2 ProjectFolderLayout: {D}/ prefix, constant OID=0, NO double-hex.
func TestProjectFolderLayout_Names(t *testing.T) {
	l := ProjectFolderLayout{}
	const d = "proj1"
	if got, want := l.DocV2Name(d), "proj1/doc_v2"; got != want {
		t.Errorf("DocV2Name = %q, want %q", got, want)
	}
	if got, want := l.CheckpointName(d), "proj1/checkpoint"; got != want {
		t.Errorf("CheckpointName = %q, want %q", got, want)
	}
	// constant OID=0 → V1 KEYSPACE_DOC 00000000 sub … = 0001 00000000 …
	const hdr = "0001" + "00000000"
	if got, want := l.DocStateName(d, 0), "proj1/"+hdr+"00"; got != want {
		t.Errorf("DocStateName = %q, want %q", got, want)
	}
	if got, want := l.UpdateName(d, 0, 5), "proj1/"+hdr+"02"+"00000005"+"00"; got != want {
		t.Errorf("UpdateName = %q, want %q", got, want)
	}
	if got, want := l.UpdatePrefix(d, 0), "proj1/"+hdr+"02"; got != want {
		t.Errorf("UpdatePrefix = %q, want %q", got, want)
	}
}

// Phase-2 path safety: malicious doc ids are rejected; ':' is allowed; no UUID gate.
func TestValidateDocIDForPrefix(t *testing.T) {
	valid := []string{
		"proj1",
		"main",
		"01234567-89ab-cdef-0123-456789abcdef",
		"a_b.c-d",
		"project:main", // ':' is allowed — opacity; we never parse it
	}
	for _, d := range valid {
		if err := ValidateDocIDForPrefix(d); err != nil {
			t.Errorf("ValidateDocIDForPrefix(%q) = %v, want nil", d, err)
		}
	}
	malicious := []string{
		"",       // empty
		"   ",    // whitespace-only
		"a/b",    // slash → traversal/forge
		"a/../b", // dot-dot segment
		"..",     // dot-dot
		".",      // dot segment
		"a/",     // trailing slash
		"/a",     // leading slash
		"a\x00b", // NUL
		"a\nb",   // control char
		"a\tb",   // control char
		" a",     // leading whitespace
		"a ",     // trailing whitespace
	}
	for _, d := range malicious {
		if err := ValidateDocIDForPrefix(d); err == nil {
			t.Errorf("ValidateDocIDForPrefix(%q) = nil, want error", d)
		}
	}
}

func TestSnapVersionNaming_LegacyRoot(t *testing.T) {
	var l LegacyRootLayout
	const room DocID = "01kykrds8s93qfnxx8g8d26hsy"

	prefix := l.SnapVersionPrefix(room)
	name := l.SnapVersionName(room, 7)

	// The listing prefix must actually be a prefix of a member object, or
	// ListSnapshots cannot enumerate. Hex encoding is per-byte, so a shared
	// string prefix yields a shared hex prefix.
	if !strings.HasPrefix(name, prefix) {
		t.Fatalf("name %q does not start with prefix %q", name, prefix)
	}
	// The id must round-trip out of the object name.
	got, ok := snapVersionID(name, prefix)
	if !ok || got != 7 {
		t.Fatalf("snapVersionID = (%d,%v), want (7,true)", got, ok)
	}
	// Distinct rooms must not share a prefix.
	if strings.HasPrefix(l.SnapVersionName("other", 7), prefix) {
		t.Fatal("a different room must not fall under this room's prefix")
	}
	// nextid must NOT fall under the listing prefix, or it would be listed as a snapshot.
	if strings.HasPrefix(l.SnapNextIDName(room), prefix) {
		t.Fatal("nextid object must not be inside the snapshot listing prefix")
	}
}

func TestSnapVersionNaming_ProjectFolder(t *testing.T) {
	var l ProjectFolderLayout
	const room DocID = "01kykrds8s93qfnxx8g8d26hsy"

	prefix := l.SnapVersionPrefix(room)
	name := l.SnapVersionName(room, 42)
	if !strings.HasPrefix(name, prefix) {
		t.Fatalf("name %q does not start with prefix %q", name, prefix)
	}
	got, ok := snapVersionID(name, prefix)
	if !ok || got != 42 {
		t.Fatalf("snapVersionID = (%d,%v), want (42,true)", got, ok)
	}
	if strings.HasPrefix(l.SnapNextIDName(room), prefix) {
		t.Fatal("nextid object must not be inside the snapshot listing prefix")
	}
}
