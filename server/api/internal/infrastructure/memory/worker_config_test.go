package memory

import (
	"context"
	"testing"

	"github.com/reearth/reearth-flow/api/pkg/workerconfig"
)

func TestWorkerConfigMemory(t *testing.T) {
	repo := NewWorkerConfig()
	ctx := context.Background()

	cfg := workerconfig.New()
	cpu := 2500
	cfg.SetComputeCpuMilli(&cpu)

	if err := repo.Save(ctx, cfg); err != nil {
		t.Fatalf("save failed: %v", err)
	}

	stored, err := repo.FindByID(ctx, cfg.ID())
	if err != nil {
		t.Fatalf("find failed: %v", err)
	}
	if stored == nil || stored.ComputeCpuMilli() == nil || *stored.ComputeCpuMilli() != cpu {
		t.Fatalf("unexpected stored config: %+v", stored)
	}

	// Test FindAll
	storedAll, err := repo.FindAll(ctx)
	if err != nil {
		t.Fatalf("find all failed: %v", err)
	}
	if storedAll == nil || storedAll.ID() != cfg.ID() {
		t.Fatalf("unexpected stored config from FindAll: %+v", storedAll)
	}

	if err := repo.Remove(ctx, cfg.ID()); err != nil {
		t.Fatalf("remove failed: %v", err)
	}

	stored, err = repo.FindByID(ctx, cfg.ID())
	if err != nil {
		t.Fatalf("find after remove failed: %v", err)
	}
	if stored != nil {
		t.Fatalf("expected nil after remove, got %+v", stored)
	}
}
