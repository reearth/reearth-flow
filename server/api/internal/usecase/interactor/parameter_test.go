package interactor

import (
	"context"
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/memory"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/parameter"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearthx/account/accountdomain/user"
	"github.com/reearth/reearthx/account/accountdomain/workspace"
	"github.com/reearth/reearthx/account/accountinfrastructure/accountmemory"
	"github.com/reearth/reearthx/appx"
	"github.com/reearth/reearthx/rerror"
	"github.com/reearth/reearthx/usecasex"
	"github.com/stretchr/testify/assert"
)

func setupParameterInteractor() (interfaces.Parameter, context.Context, *repo.Container, id.ProjectID) {
	mockAuthInfo := &appx.AuthInfo{
		Token: "token",
	}
	mockUser := user.New().NewID().Name("hoge").Email("abc@bb.cc").MustBuild()

	ctx := context.Background()
	ctx = adapter.AttachAuthInfo(ctx, mockAuthInfo)
	ctx = adapter.AttachReearthxUser(ctx, mockUser)

	paramRepo := memory.NewParameter()
	projectRepo := memory.NewProject()
	workspaceRepo := accountmemory.NewWorkspace()

	r := &repo.Container{
		Parameter:   paramRepo,
		Project:     projectRepo,
		Transaction: &usecasex.NopTransaction{},
	}

	ws := workspace.New().NewID().MustBuild()
	_ = workspaceRepo.Save(ctx, ws)

	pid := project.NewID()
	defer project.MockNewID(pid)()
	prj := project.New().ID(pid).Workspace(ws.ID()).Name("testproject").UpdatedAt(time.Now()).MustBuild()
	_ = projectRepo.Save(ctx, prj)

	mockPermissionCheckerTrue := NewMockPermissionChecker(func(ctx context.Context, authInfo *appx.AuthInfo, userId, resource, action string) (bool, error) {
		return true, nil
	})

	i := NewParameter(r, mockPermissionCheckerTrue)
	return i, ctx, r, pid
}

func TestParameter_DeclareParameter(t *testing.T) {
	i, ctx, _, pid := setupParameterInteractor()

	name := "param1"
	typ := parameter.TypeText
	val := "initial value"
	req := true
	pub := false
	p, err := i.DeclareParameter(ctx, interfaces.DeclareParameterParam{
		ProjectID:    pid,
		Name:         name,
		Type:         typ,
		Required:     req,
		Public:       pub,
		DefaultValue: val,
		Index:        nil,
	})
	assert.NoError(t, err)
	assert.NotNil(t, p)
	assert.Equal(t, name, p.Name())
	assert.Equal(t, typ, p.Type())
	assert.Equal(t, req, p.Required())
	assert.Equal(t, pub, p.Public())
	assert.Equal(t, val, p.DefaultValue())
	assert.Equal(t, 0, p.Index()) // first parameter gets index 0
}

func TestParameter_DeclareParameter_NonexistentProject(t *testing.T) {
	i, ctx, _, _ := setupParameterInteractor()

	// Use a random project ID that doesn't exist
	nonexistentPID := id.NewProjectID()
	p, err := i.DeclareParameter(ctx, interfaces.DeclareParameterParam{
		ProjectID:    nonexistentPID,
		Name:         "param2",
		Type:         parameter.TypeNumber,
		DefaultValue: 123,
	})
	assert.Nil(t, p)
	assert.Same(t, rerror.ErrNotFound, err)
}

func TestParameter_Fetch(t *testing.T) {
	i, ctx, _, pid := setupParameterInteractor()

	// Create a few parameters
	p1, err := i.DeclareParameter(ctx, interfaces.DeclareParameterParam{
		ProjectID:    pid,
		Name:         "param1",
		Type:         parameter.TypeText,
		DefaultValue: "val1",
	})
	assert.NoError(t, err)

	p2, err := i.DeclareParameter(ctx, interfaces.DeclareParameterParam{
		ProjectID:    pid,
		Name:         "param2",
		Type:         parameter.TypeNumber,
		DefaultValue: 42,
	})
	assert.NoError(t, err)

	params, err := i.Fetch(ctx, id.ParameterIDList{p1.ID(), p2.ID()})
	assert.NoError(t, err)
	assert.NotNil(t, params)
	assert.Len(t, *params, 2)
}

func TestParameter_FetchByProject(t *testing.T) {
	i, ctx, _, pid := setupParameterInteractor()

	// Create parameters under the project
	_, err := i.DeclareParameter(ctx, interfaces.DeclareParameterParam{
		ProjectID:    pid,
		Name:         "param1",
		Type:         parameter.TypeText,
		DefaultValue: "val1",
	})
	assert.NoError(t, err)

	_, err = i.DeclareParameter(ctx, interfaces.DeclareParameterParam{
		ProjectID:    pid,
		Name:         "param2",
		Type:         parameter.TypeNumber,
		DefaultValue: 100,
	})
	assert.NoError(t, err)

	params, err := i.FetchByProject(ctx, pid)
	assert.NoError(t, err)
	assert.NotNil(t, params)
	assert.Len(t, *params, 2)
}

func TestParameter_RemoveParameter(t *testing.T) {
	i, ctx, _, pid := setupParameterInteractor()

	// Create parameters for removal test
	p1, err := i.DeclareParameter(ctx, interfaces.DeclareParameterParam{
		ProjectID:    pid,
		Name:         "param1",
		Type:         parameter.TypeText,
		DefaultValue: "val1",
	})
	assert.NoError(t, err)

	p2, err := i.DeclareParameter(ctx, interfaces.DeclareParameterParam{
		ProjectID:    pid,
		Name:         "param2",
		Type:         parameter.TypeText,
		DefaultValue: "val2",
	})
	assert.NoError(t, err)

	// Remove param1
	removedID, err := i.RemoveParameter(ctx, p1.ID())
	assert.NoError(t, err)
	assert.Equal(t, p1.ID(), removedID)

	// Check that only p2 remains, and its index has been updated if needed
	params, err := i.FetchByProject(ctx, pid)
	assert.NoError(t, err)
	assert.Len(t, *params, 1)
	assert.Equal(t, p2.ID(), (*params)[0].ID())
	assert.Equal(t, 0, (*params)[0].Index())
}

func TestParameter_UpdateParameterOrder(t *testing.T) {
	i, ctx, _, pid := setupParameterInteractor()

	// Create parameters in some order: param1 (0), param2 (1), param3 (2)
	p1, err := i.DeclareParameter(ctx, interfaces.DeclareParameterParam{
		ProjectID:    pid,
		Name:         "param1",
		Type:         parameter.TypeText,
		DefaultValue: "val1",
	})
	assert.NoError(t, err)

	p2, err := i.DeclareParameter(ctx, interfaces.DeclareParameterParam{
		ProjectID:    pid,
		Name:         "param2",
		Type:         parameter.TypeText,
		DefaultValue: "val2",
	})
	assert.NoError(t, err)

	p3, err := i.DeclareParameter(ctx, interfaces.DeclareParameterParam{
		ProjectID:    pid,
		Name:         "param3",
		Type:         parameter.TypeText,
		DefaultValue: "val3",
	})
	assert.NoError(t, err)

	// Move param3 to the front (index 0)
	updatedParams, err := i.UpdateParameterOrder(ctx, interfaces.UpdateParameterOrderParam{
		ProjectID: pid,
		ParamID:   p3.ID(),
		NewIndex:  0,
	})
	assert.NoError(t, err)
	assert.NotNil(t, updatedParams)

	// Check new order: p3(0), p1(1), p2(2)
	p3u := updatedParams.FindByID(p3.ID())
	p1u := updatedParams.FindByID(p1.ID())
	p2u := updatedParams.FindByID(p2.ID())
	assert.NotNil(t, p3u)
	assert.NotNil(t, p1u)
	assert.NotNil(t, p2u)
	assert.Equal(t, 0, p3u.Index())
	assert.Equal(t, 1, p1u.Index())
	assert.Equal(t, 2, p2u.Index())
}

func TestParameter_UpdateParameter(t *testing.T) {
	i, ctx, _, pid := setupParameterInteractor()

	p1, err := i.DeclareParameter(ctx, interfaces.DeclareParameterParam{
		ProjectID:    pid,
		Name:         "param1",
		Type:         parameter.TypeText,
		DefaultValue: "old value",
	})
	assert.NoError(t, err)

	// Update param1's value
	newVal := "new value"
	updatedParam, err := i.UpdateParameter(ctx, interfaces.UpdateParameterParam{
		ParamID:      p1.ID(),
		DefaultValue: newVal,
	})
	assert.NoError(t, err)
	assert.NotNil(t, updatedParam)
	assert.Equal(t, newVal, updatedParam.DefaultValue())
}

func TestParameter_UpdateParameterValue_NotFound(t *testing.T) {
	i, ctx, _, _ := setupParameterInteractor()

	// Try updating a parameter that does not exist
	nonexistentParamID := id.NewParameterID()
	updatedParam, err := i.UpdateParameter(ctx, interfaces.UpdateParameterParam{
		ParamID:      nonexistentParamID,
		DefaultValue: "something",
	})
	assert.Nil(t, updatedParam)
	assert.Same(t, rerror.ErrNotFound, err)
}

func TestParameter_RemoveParameters(t *testing.T) {
	i, ctx, _, pid := setupParameterInteractor()

	// Create multiple parameters for batch removal test
	p1, err := i.DeclareParameter(ctx, interfaces.DeclareParameterParam{
		ProjectID:    pid,
		Name:         "param1",
		Type:         parameter.TypeText,
		DefaultValue: "val1",
	})
	assert.NoError(t, err)

	p2, err := i.DeclareParameter(ctx, interfaces.DeclareParameterParam{
		ProjectID:    pid,
		Name:         "param2",
		Type:         parameter.TypeText,
		DefaultValue: "val2",
	})
	assert.NoError(t, err)

	p3, err := i.DeclareParameter(ctx, interfaces.DeclareParameterParam{
		ProjectID:    pid,
		Name:         "param3",
		Type:         parameter.TypeText,
		DefaultValue: "val3",
	})
	assert.NoError(t, err)

	p4, err := i.DeclareParameter(ctx, interfaces.DeclareParameterParam{
		ProjectID:    pid,
		Name:         "param4",
		Type:         parameter.TypeText,
		DefaultValue: "val4",
	})
	assert.NoError(t, err)

	// Remove p1 and p3 (non-consecutive parameters)
	removedIDs, err := i.RemoveParameters(ctx, id.ParameterIDList{p1.ID(), p3.ID()})
	assert.NoError(t, err)
	assert.Len(t, removedIDs, 2)
	assert.Contains(t, removedIDs, p1.ID())
	assert.Contains(t, removedIDs, p3.ID())

	// Check that only p2 and p4 remain with properly recalculated indexes
	params, err := i.FetchByProject(ctx, pid)
	assert.NoError(t, err)
	assert.Len(t, *params, 2)

	// Find remaining parameters
	var remainingP2, remainingP4 *parameter.Parameter
	for _, p := range *params {
		if p.ID() == p2.ID() {
			remainingP2 = p
		} else if p.ID() == p4.ID() {
			remainingP4 = p
		}
	}

	assert.NotNil(t, remainingP2)
	assert.NotNil(t, remainingP4)

	// Check that indexes are recalculated sequentially (0, 1)
	assert.Equal(t, 0, remainingP2.Index())
	assert.Equal(t, 1, remainingP4.Index())
}

func TestParameter_RemoveParameters_EmptyList(t *testing.T) {
	i, ctx, _, _ := setupParameterInteractor()

	// Remove empty list should succeed
	removedIDs, err := i.RemoveParameters(ctx, id.ParameterIDList{})
	assert.NoError(t, err)
	assert.Len(t, removedIDs, 0)
}

func TestParameter_RemoveParameters_NotFound(t *testing.T) {
	i, ctx, _, _ := setupParameterInteractor()

	// Try removing parameters that don't exist
	nonexistentID1 := id.NewParameterID()
	nonexistentID2 := id.NewParameterID()
	removedIDs, err := i.RemoveParameters(ctx, id.ParameterIDList{nonexistentID1, nonexistentID2})
	assert.Nil(t, removedIDs)
	assert.Same(t, rerror.ErrNotFound, err)
}

func TestParameter_UpdateParameters(t *testing.T) {
	i, ctx, _, pid := setupParameterInteractor()

	// Create some initial parameters
	p1, err := i.DeclareParameter(ctx, interfaces.DeclareParameterParam{
		ProjectID:    pid,
		Name:         "param1",
		Type:         parameter.TypeText,
		DefaultValue: "val1",
		Required:     true,
		Public:       false,
	})
	assert.NoError(t, err)

	p2, err := i.DeclareParameter(ctx, interfaces.DeclareParameterParam{
		ProjectID:    pid,
		Name:         "param2",
		Type:         parameter.TypeNumber,
		DefaultValue: 42,
		Required:     false,
		Public:       true,
	})
	assert.NoError(t, err)

	// Test batch operations: create, update, delete, reorder
	newName := "updatedParam1"
	newType := parameter.TypePassword
	newRequired := false
	newPublic := true

	result, err := i.UpdateParameters(ctx, interfaces.UpdateParametersParam{
		ProjectID: pid,
		Creates: []interfaces.DeclareParameterParam{
			{
				ProjectID:    pid,
				Name:         "newParam",
				Type:         parameter.TypeColor,
				DefaultValue: "#FF0000",
				Required:     true,
				Public:       false,
			},
		},
		Updates: []interfaces.UpdateParameterBatchItemParam{
			{
				ParamID:       p1.ID(),
				NameValue:     &newName,
				TypeValue:     &newType,
				RequiredValue: &newRequired,
				PublicValue:   &newPublic,
				DefaultValue:  "newDefaultValue",
			},
		},
		Deletes: id.ParameterIDList{p2.ID()},
		Reorders: []interfaces.UpdateParameterOrderParam{
			{
				ParamID:   p1.ID(),
				NewIndex:  1,
				ProjectID: pid,
			},
		},
	})

	assert.NoError(t, err)
	assert.NotNil(t, result)
	assert.Len(t, *result, 2) // p1 (updated) + newParam

	// Verify the results
	updatedP1 := result.FindByID(p1.ID())
	assert.NotNil(t, updatedP1)
	assert.Equal(t, newName, updatedP1.Name())
	assert.Equal(t, newType, updatedP1.Type())
	assert.Equal(t, newRequired, updatedP1.Required())
	assert.Equal(t, newPublic, updatedP1.Public())
	assert.Equal(t, "newDefaultValue", updatedP1.DefaultValue())
	assert.Equal(t, 1, updatedP1.Index()) // Reordered to index 1

	// Find the new parameter
	var newParam *parameter.Parameter
	for _, p := range *result {
		if p.Name() == "newParam" {
			newParam = p
			break
		}
	}
	assert.NotNil(t, newParam)
	assert.Equal(t, parameter.TypeColor, newParam.Type())
	assert.Equal(t, "#FF0000", newParam.DefaultValue())
	assert.Equal(t, 0, newParam.Index()) // Should be at index 0

	// Verify p2 was deleted (should not be in result)
	deletedP2 := result.FindByID(p2.ID())
	assert.Nil(t, deletedP2)
}

func TestParameter_UpdateParameters_PartialUpdates(t *testing.T) {
	i, ctx, _, pid := setupParameterInteractor()

	// Create a parameter
	p1, err := i.DeclareParameter(ctx, interfaces.DeclareParameterParam{
		ProjectID:    pid,
		Name:         "param1",
		Type:         parameter.TypeText,
		DefaultValue: "originalValue",
		Required:     true,
		Public:       false,
	})
	assert.NoError(t, err)

	// Test partial update (only update name and defaultValue)
	newName := "updatedName"
	result, err := i.UpdateParameters(ctx, interfaces.UpdateParametersParam{
		ProjectID: pid,
		Updates: []interfaces.UpdateParameterBatchItemParam{
			{
				ParamID:      p1.ID(),
				NameValue:    &newName,
				DefaultValue: "newValue",
				// Don't provide Type, Required, Public - should preserve original values
			},
		},
	})

	assert.NoError(t, err)
	assert.NotNil(t, result)
	assert.Len(t, *result, 1)

	updatedP1 := result.FindByID(p1.ID())
	assert.NotNil(t, updatedP1)
	assert.Equal(t, newName, updatedP1.Name())
	assert.Equal(t, "newValue", updatedP1.DefaultValue())
	// These should be preserved from original
	assert.Equal(t, parameter.TypeText, updatedP1.Type())
	assert.Equal(t, true, updatedP1.Required())
	assert.Equal(t, false, updatedP1.Public())
}

func TestParameter_UpdateParameters_EmptyOperations(t *testing.T) {
	i, ctx, _, pid := setupParameterInteractor()

	// Test with no operations
	result, err := i.UpdateParameters(ctx, interfaces.UpdateParametersParam{
		ProjectID: pid,
	})

	assert.NoError(t, err)
	assert.NotNil(t, result)
	assert.Len(t, *result, 0) // No parameters in project
}

func TestParameter_UpdateParameters_NonexistentProject(t *testing.T) {
	i, ctx, _, _ := setupParameterInteractor()

	nonexistentPID := id.NewProjectID()
	result, err := i.UpdateParameters(ctx, interfaces.UpdateParametersParam{
		ProjectID: nonexistentPID,
		Creates: []interfaces.DeclareParameterParam{
			{
				ProjectID:    nonexistentPID,
				Name:         "param1",
				Type:         parameter.TypeText,
				DefaultValue: "value",
			},
		},
	})

	assert.Nil(t, result)
	assert.Same(t, rerror.ErrNotFound, err)
}
