package interactor

import (
	"context"
	"errors"
	"testing"

	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/stretchr/testify/assert"
)

func TestCheckPermission(t *testing.T) {
	checkerAllow := NewMockPermissionChecker(func(_ context.Context, _, _ string) (bool, error) {
		return true, nil
	})
	checkerDeny := NewMockPermissionChecker(func(_ context.Context, _, _ string) (bool, error) {
		return false, nil
	})
	checkerErr := NewMockPermissionChecker(func(_ context.Context, _, _ string) (bool, error) {
		return false, errors.New("service unavailable")
	})

	tests := []struct {
		ctx     context.Context
		checker *mockPermissionChecker
		wantErr error
		name    string
	}{
		{
			name:    "grants permission when checker allows",
			ctx:     context.Background(),
			checker: checkerAllow,
			wantErr: nil,
		},
		{
			name:    "denies when checker returns false",
			ctx:     context.Background(),
			checker: checkerDeny,
			wantErr: interfaces.ErrOperationDenied,
		},
		{
			name:    "propagates error from checker",
			ctx:     context.Background(),
			checker: checkerErr,
			wantErr: errors.New("service unavailable"),
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			setSkipPermissionCheck(false)
			err := checkPermission(tt.ctx, tt.checker, "project", "create")
			if tt.wantErr == nil {
				assert.NoError(t, err)
			} else {
				assert.EqualError(t, err, tt.wantErr.Error())
			}
		})
	}

	t.Run("skips check entirely when skipPermissionCheck is true", func(t *testing.T) {
		setSkipPermissionCheck(true)
		defer setSkipPermissionCheck(false)
		err := checkPermission(context.Background(), checkerDeny, "project", "delete")
		assert.NoError(t, err)
	})
}
