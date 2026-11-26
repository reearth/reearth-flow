package gcs

import "strconv"

// coverage is what doc_v2 holds, recorded as object metadata on every write so
// a rebuild can tell whether the base already runs past a version.
type coverage struct {
	// clock is the highest update the base holds; every clock at or below it is in.
	clock uint32
	// live means the base was built from a live room, which can hold edits that
	// no update object carries yet, so it may also run past clock.
	live bool
}

const (
	metaCovers = "covers"
	metaLive   = "live"
)

func (c coverage) meta() map[string]string {
	m := map[string]string{metaCovers: strconv.FormatUint(uint64(c.clock), 10)}
	if c.live {
		m[metaLive] = "1"
	}
	return m
}

// coverageOf reads coverage from doc_v2's metadata, reporting false for a doc_v2
// written before coverage was recorded.
func coverageOf(meta map[string]string) (coverage, bool) {
	s, ok := meta[metaCovers]
	if !ok {
		return coverage{}, false
	}
	n, err := strconv.ParseUint(s, 10, 32)
	if err != nil {
		return coverage{}, false
	}
	return coverage{clock: uint32(n), live: meta[metaLive] == "1"}, true
}
