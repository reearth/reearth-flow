package gcs

// Logarithmic retention thinning. A pure helper (not wired to any hot path),
// exposed so the thinning math can back an optional Compact policy.

// firstZeroBit returns the lowest unset bit of x as a power of two: (x+1) & ^x.
func firstZeroBit(x uint32) uint32 {
	return (x + 1) &^ x
}

// trimUpdatesLogarithmic returns the set of update clocks to retain after
// appending clocks 1..=n: at each n it drops clock n-(firstZeroBit(n)<<densityShift)
// when that offset is below n. Yields ~2·log2(n) survivors, always keeping n.
// Deterministic and side-effect-free.
func trimUpdatesLogarithmic(n, densityShift uint32) map[uint32]struct{} {
	retained := make(map[uint32]struct{})
	for clk := uint32(1); clk <= n; clk++ {
		retained[clk] = struct{}{}
		bit := firstZeroBit(clk)
		deleteOffset := bit << densityShift
		if deleteOffset < clk {
			delete(retained, clk-deleteOffset)
		}
	}
	return retained
}
