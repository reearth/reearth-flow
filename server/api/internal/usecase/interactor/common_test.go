package interactor

import (
	"context"
	"errors"
	"testing"

	accountsuser "github.com/reearth/reearth-accounts/server/pkg/user"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearthx/appx"
	"github.com/stretchr/testify/assert"
)

func TestCheckPermission(t *testing.T) {
	validAuthInfo := &appx.AuthInfo{Token: "token"}
	validUser := accountsuser.New().NewID().Name("test").Email("test@example.com").MustBuild()

	checkerAllow := NewMockPermissionChecker(func(_ context.Context, _ *appx.AuthInfo, _, _, _ string) (bool, error) {
		return true, nil
	})
	checkerDeny := NewMockPermissionChecker(func(_ context.Context, _ *appx.AuthInfo, _, _, _ string) (bool, error) {
		return false, nil
	})
	checkerErr := NewMockPermissionChecker(func(_ context.Context, _ *appx.AuthInfo, _, _, _ string) (bool, error) {
		return false, errors.New("service unavailable")
	})

	baseCtx := func() context.Context {
		ctx := context.Background()
		ctx = adapter.AttachAuthInfo(ctx, validAuthInfo)
		ctx = adapter.AttachUser(ctx, validUser)
		return ctx
	}

	tests := []struct {
		ctx     context.Context
		checker *mockPermissionChecker
		wantErr error
		name    string
	}{
		{
			name:    "grants permission when checker allows",
			ctx:     baseCtx(),
			checker: checkerAllow,
			wantErr: nil,
		},
		{
			name:    "denies when checker returns false",
			ctx:     baseCtx(),
			checker: checkerDeny,
			wantErr: interfaces.ErrOperationDenied,
		},
		{
			name:    "denies when checker returns error",
			ctx:     baseCtx(),
			checker: checkerErr,
			wantErr: errors.New("service unavailable"),
		},
		{
			name:    "uses JWT from context when AuthInfo is not set",
			ctx:     adapter.AttachJWT(adapter.AttachUser(context.Background(), validUser), "jwt-token"),
			checker: checkerAllow,
			wantErr: nil,
		},
		{
			name:    "denies when user is missing from context",
			ctx:     adapter.AttachAuthInfo(context.Background(), validAuthInfo),
			checker: checkerAllow,
			wantErr: interfaces.ErrOperationDenied,
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
