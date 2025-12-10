package memory

import (
	"context"
	"testing"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/workerconfig"
)

func TestWorkerConfigMemory(t *testing.T) {
	repo := NewWorkerConfig()
	ctx := context.Background()
	ws := id.NewWorkspaceID()

	cfg := workerconfig.New(ws)
	cpu := 2500
	cfg.SetComputeCpuMilli(&cpu)

	if err := repo.Save(ctx, cfg); err != nil {
		t.Fatalf("save failed: %v", err)
	}

	stored, err := repo.FindByWorkspace(ctx, ws)
	if err != nil {
		t.Fatalf("find failed: %v", err)
	}
	if stored == nil || stored.ComputeCpuMilli() == nil || *stored.ComputeCpuMilli() != cpu {
		t.Fatalf("unexpected stored config: %+v", stored)
	}

	if err := repo.Remove(ctx, ws); err != nil {
		t.Fatalf("remove failed: %v", err)
	}

	stored, err = repo.FindByWorkspace(ctx, ws)
	if err != nil {
		t.Fatalf("find after remove failed: %v", err)
	}
	if stored != nil {
		t.Fatalf("expected nil after remove, got %+v", stored)
	}
}
