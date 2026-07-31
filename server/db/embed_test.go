package db

import (
	"io/fs"
	"os"
	"strings"
	"testing"
)

// The embed must stay in sync with the on-disk migrations dir. A missing
// embed glob would silently ship an incomplete schema, so assert the
// embedded .sql set exactly equals what Atlas wrote to disk.
func TestMigrationsFS_MatchesDisk(t *testing.T) {
	embedded := map[string]bool{}
	entries, err := fs.ReadDir(MigrationsFS, "migrations")
	if err != nil {
		t.Fatalf("read embedded migrations: %v", err)
	}
	for _, e := range entries {
		if strings.HasSuffix(e.Name(), ".sql") {
			embedded[e.Name()] = true
		}
	}

	disk, err := os.ReadDir("migrations")
	if err != nil {
		t.Fatalf("read disk migrations: %v", err)
	}
	var want int
	for _, e := range disk {
		if !strings.HasSuffix(e.Name(), ".sql") {
			continue
		}
		want++
		if !embedded[e.Name()] {
			t.Errorf("migration %s is on disk but not embedded", e.Name())
		}
	}
	if want == 0 {
		t.Fatal("no .sql migrations found on disk")
	}
	if len(embedded) != want {
		t.Errorf("embedded %d migrations, disk has %d", len(embedded), want)
	}
}
