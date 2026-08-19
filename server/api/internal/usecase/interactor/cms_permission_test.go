package interactor

import (
	"context"
	"errors"
	"testing"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/pkg/cms"
	"github.com/reearth/reearthx/rerror"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// fakeCMSGateway is an in-memory gateway.CMS keyed by project/model/asset id,
// with call counters so tests can assert denied checks never reach it.
type fakeCMSGateway struct {
	projects map[string]*cms.Project
	assets   map[string]*cms.Asset
	models   map[string]*cms.Model

	getProjectCalls               int
	listProjectsCalls             int
	getAssetCalls                 int
	listAssetsCalls               int
	getModelCalls                 int
	listModelsCalls               int
	listItemsCalls                int
	getModelExportURLCalls        int
	getModelGeoJSONExportURLCalls int
}

var _ gateway.CMS = (*fakeCMSGateway)(nil)

func (f *fakeCMSGateway) GetProject(_ context.Context, projectIDOrAlias string) (*cms.Project, error) {
	f.getProjectCalls++
	if p, ok := f.projects[projectIDOrAlias]; ok {
		return p, nil
	}
	return nil, nil
}

func (f *fakeCMSGateway) ListProjects(_ context.Context, _ cms.ListProjectsInput) (*cms.ListProjectsOutput, error) {
	f.listProjectsCalls++
	return &cms.ListProjectsOutput{}, nil
}

func (f *fakeCMSGateway) GetAsset(_ context.Context, input cms.GetAssetInput) (*cms.Asset, error) {
	f.getAssetCalls++
	if a, ok := f.assets[input.AssetID]; ok {
		return a, nil
	}
	return nil, nil
}

func (f *fakeCMSGateway) ListAssets(_ context.Context, _ cms.ListAssetsInput) (*cms.ListAssetsOutput, error) {
	f.listAssetsCalls++
	return &cms.ListAssetsOutput{}, nil
}

func (f *fakeCMSGateway) GetModel(_ context.Context, input cms.GetModelInput) (*cms.Model, error) {
	f.getModelCalls++
	if m, ok := f.models[input.ProjectIDOrAlias+"/"+input.ModelIDOrAlias]; ok {
		return m, nil
	}
	return nil, nil
}

func (f *fakeCMSGateway) ListModels(_ context.Context, _ cms.ListModelsInput) (*cms.ListModelsOutput, error) {
	f.listModelsCalls++
	return &cms.ListModelsOutput{}, nil
}

func (f *fakeCMSGateway) ListItems(_ context.Context, _ cms.ListItemsInput) (*cms.ListItemsOutput, error) {
	f.listItemsCalls++
	return &cms.ListItemsOutput{}, nil
}

func (f *fakeCMSGateway) GetModelExportURL(_ context.Context, _ cms.ModelExportInput) (*cms.ExportOutput, error) {
	f.getModelExportURLCalls++
	return &cms.ExportOutput{URL: "https://example.com/export.json"}, nil
}

func (f *fakeCMSGateway) GetModelGeoJSONExportURL(_ context.Context, _ cms.ExportInput) (*cms.ExportOutput, error) {
	f.getModelGeoJSONExportURLCalls++
	return &cms.ExportOutput{URL: "https://example.com/export.geojson"}, nil
}

// sequenceChecker records the workspace ids of every CheckPermission call, in order.
type sequenceChecker struct {
	calls  [][]accountsid.WorkspaceID
	allow  bool
	action string
}

func (s *sequenceChecker) CheckPermission(_ context.Context, _, action string, workspaceID ...accountsid.WorkspaceID) (bool, error) {
	s.calls = append(s.calls, workspaceID)
	s.action = action
	return s.allow, nil
}

func newCMSFixture(cmsGW *fakeCMSGateway, rc gateway.PermissionChecker) *cmsInteractor {
	return &cmsInteractor{
		gateways:          &gateway.Container{CMS: cmsGW},
		permissionChecker: rc,
	}
}

func TestCMS_GetCMSProject_ChecksOwningWorkspace(t *testing.T) {
	wsA := accountsid.NewWorkspaceID()
	gw := &fakeCMSGateway{projects: map[string]*cms.Project{
		"p1": {ID: "p1", WorkspaceID: wsA.String()},
	}}
	rc := &recordingChecker{allow: true}
	i := newCMSFixture(gw, rc)

	proj, err := i.GetCMSProject(context.Background(), "p1")

	require.NoError(t, err)
	require.NotNil(t, proj)
	assert.Equal(t, rbac.ResourceCMSProject, rc.gotResource)
	assert.Equal(t, rbac.ActionAny, rc.gotAction)
	require.Len(t, rc.gotWorkspace, 1)
	assert.Equal(t, wsA, rc.gotWorkspace[0])
}

func TestCMS_GetCMSProject_DeniedReturnsNotFound(t *testing.T) {
	wsA := accountsid.NewWorkspaceID()
	gw := &fakeCMSGateway{projects: map[string]*cms.Project{
		"p1": {ID: "p1", WorkspaceID: wsA.String()},
	}}
	rc := &recordingChecker{allow: false}
	i := newCMSFixture(gw, rc)

	proj, err := i.GetCMSProject(context.Background(), "p1")

	assert.Nil(t, proj)
	assert.ErrorIs(t, err, rerror.ErrNotFound)
	require.Len(t, rc.gotWorkspace, 1)
	assert.Equal(t, wsA, rc.gotWorkspace[0])
}

func TestCMS_ListCMSProjects_ChecksEachRequestedWorkspace(t *testing.T) {
	wsA := accountsid.NewWorkspaceID()
	wsB := accountsid.NewWorkspaceID()
	gw := &fakeCMSGateway{}
	sc := &sequenceChecker{allow: true}
	i := newCMSFixture(gw, sc)

	out, err := i.ListCMSProjects(context.Background(), []string{wsA.String(), wsB.String()}, nil, false, nil, nil)

	require.NoError(t, err)
	require.NotNil(t, out)
	require.Len(t, sc.calls, 2)
	require.Len(t, sc.calls[0], 1)
	require.Len(t, sc.calls[1], 1)
	assert.Equal(t, wsA, sc.calls[0][0])
	assert.Equal(t, wsB, sc.calls[1][0])
	assert.Equal(t, 1, gw.listProjectsCalls)
}

func TestCMS_ListCMSProjects_SecondWorkspaceDeniedDeniesWholeCall(t *testing.T) {
	wsA := accountsid.NewWorkspaceID()
	wsB := accountsid.NewWorkspaceID()
	gw := &fakeCMSGateway{}
	rc := &perWorkspaceChecker{denied: wsB}
	i := newCMSFixture(gw, rc)

	out, err := i.ListCMSProjects(context.Background(), []string{wsA.String(), wsB.String()}, nil, false, nil, nil)

	assert.Nil(t, out)
	assert.ErrorIs(t, err, rerror.ErrNotFound)
	assert.Zero(t, gw.listProjectsCalls, "denied call must not reach the CMS gateway")
}

func TestCMS_GetCMSAsset_ChecksOwningWorkspace(t *testing.T) {
	wsA := accountsid.NewWorkspaceID()
	gw := &fakeCMSGateway{
		projects: map[string]*cms.Project{"p1": {ID: "p1", WorkspaceID: wsA.String()}},
		assets:   map[string]*cms.Asset{"a1": {ID: "a1", ProjectID: "p1"}},
	}
	rc := &recordingChecker{allow: true}
	i := newCMSFixture(gw, rc)

	asset, err := i.GetCMSAsset(context.Background(), "a1")

	require.NoError(t, err)
	require.NotNil(t, asset)
	assert.Equal(t, rbac.ResourceCMSAsset, rc.gotResource)
	require.Len(t, rc.gotWorkspace, 1)
	assert.Equal(t, wsA, rc.gotWorkspace[0])
}

func TestCMS_GetCMSAsset_DeniedReturnsNotFound(t *testing.T) {
	wsA := accountsid.NewWorkspaceID()
	gw := &fakeCMSGateway{
		projects: map[string]*cms.Project{"p1": {ID: "p1", WorkspaceID: wsA.String()}},
		assets:   map[string]*cms.Asset{"a1": {ID: "a1", ProjectID: "p1"}},
	}
	rc := &recordingChecker{allow: false}
	i := newCMSFixture(gw, rc)

	asset, err := i.GetCMSAsset(context.Background(), "a1")

	assert.Nil(t, asset, "denied caller must not receive asset data")
	assert.ErrorIs(t, err, rerror.ErrNotFound)
	require.Len(t, rc.gotWorkspace, 1)
	assert.Equal(t, wsA, rc.gotWorkspace[0])
}

func TestCMS_ListCMSAssets_ChecksOwningWorkspaceAndDeniesFailClosed(t *testing.T) {
	wsA := accountsid.NewWorkspaceID()
	otherWS := accountsid.NewWorkspaceID()

	t.Run("allowed", func(t *testing.T) {
		gw := &fakeCMSGateway{projects: map[string]*cms.Project{"p1": {ID: "p1", WorkspaceID: wsA.String()}}}
		rc := &recordingChecker{allow: true}
		i := newCMSFixture(gw, rc)

		out, err := i.ListCMSAssets(context.Background(), "p1", nil, nil)

		require.NoError(t, err)
		require.NotNil(t, out)
		assert.Equal(t, rbac.ResourceCMSAsset, rc.gotResource)
		require.Len(t, rc.gotWorkspace, 1)
		assert.Equal(t, wsA, rc.gotWorkspace[0])
		assert.Equal(t, 1, gw.listAssetsCalls)
	})

	t.Run("denied", func(t *testing.T) {
		gw := &fakeCMSGateway{projects: map[string]*cms.Project{"p1": {ID: "p1", WorkspaceID: wsA.String()}}}
		rc := &recordingChecker{allow: false}
		i := newCMSFixture(gw, rc)

		out, err := i.ListCMSAssets(context.Background(), "p1", nil, nil)

		assert.Nil(t, out)
		assert.ErrorIs(t, err, rerror.ErrNotFound)
		assert.Zero(t, gw.listAssetsCalls, "denied call must not reach ListAssets")
	})

	t.Run("caller-supplied workspace is ignored, owning workspace is used", func(t *testing.T) {
		gw := &fakeCMSGateway{projects: map[string]*cms.Project{"p1": {ID: "p1", WorkspaceID: wsA.String()}}}
		rc := &perWorkspaceChecker{denied: otherWS}
		i := newCMSFixture(gw, rc)

		_, err := i.ListCMSAssets(context.Background(), "p1", nil, nil)
		require.NoError(t, err, "checker only denies otherWS; the owning workspace wsA must be the one checked")
	})
}

func TestCMS_GetCMSModel_ChecksOwningWorkspaceAndDeniesFailClosed(t *testing.T) {
	wsA := accountsid.NewWorkspaceID()

	t.Run("allowed", func(t *testing.T) {
		gw := &fakeCMSGateway{
			projects: map[string]*cms.Project{"p1": {ID: "p1", WorkspaceID: wsA.String()}},
			models:   map[string]*cms.Model{"p1/m1": {ID: "m1", ProjectID: "p1"}},
		}
		rc := &recordingChecker{allow: true}
		i := newCMSFixture(gw, rc)

		model, err := i.GetCMSModel(context.Background(), "p1", "m1")

		require.NoError(t, err)
		require.NotNil(t, model)
		assert.Equal(t, rbac.ResourceCMSModel, rc.gotResource)
		require.Len(t, rc.gotWorkspace, 1)
		assert.Equal(t, wsA, rc.gotWorkspace[0])
		assert.Equal(t, 1, gw.getModelCalls)
	})

	t.Run("denied", func(t *testing.T) {
		gw := &fakeCMSGateway{
			projects: map[string]*cms.Project{"p1": {ID: "p1", WorkspaceID: wsA.String()}},
			models:   map[string]*cms.Model{"p1/m1": {ID: "m1", ProjectID: "p1"}},
		}
		rc := &recordingChecker{allow: false}
		i := newCMSFixture(gw, rc)

		model, err := i.GetCMSModel(context.Background(), "p1", "m1")

		assert.Nil(t, model)
		assert.ErrorIs(t, err, rerror.ErrNotFound)
		assert.Zero(t, gw.getModelCalls, "denied call must not reach GetModel")
	})
}

func TestCMS_ListCMSModels_ChecksOwningWorkspaceAndDeniesFailClosed(t *testing.T) {
	wsA := accountsid.NewWorkspaceID()

	t.Run("allowed", func(t *testing.T) {
		gw := &fakeCMSGateway{projects: map[string]*cms.Project{"p1": {ID: "p1", WorkspaceID: wsA.String()}}}
		rc := &recordingChecker{allow: true}
		i := newCMSFixture(gw, rc)

		out, err := i.ListCMSModels(context.Background(), "p1", nil, nil)

		require.NoError(t, err)
		require.NotNil(t, out)
		assert.Equal(t, rbac.ResourceCMSModel, rc.gotResource)
		require.Len(t, rc.gotWorkspace, 1)
		assert.Equal(t, wsA, rc.gotWorkspace[0])
		assert.Equal(t, 1, gw.listModelsCalls)
	})

	t.Run("denied", func(t *testing.T) {
		gw := &fakeCMSGateway{projects: map[string]*cms.Project{"p1": {ID: "p1", WorkspaceID: wsA.String()}}}
		rc := &recordingChecker{allow: false}
		i := newCMSFixture(gw, rc)

		out, err := i.ListCMSModels(context.Background(), "p1", nil, nil)

		assert.Nil(t, out)
		assert.ErrorIs(t, err, rerror.ErrNotFound)
		assert.Zero(t, gw.listModelsCalls, "denied call must not reach ListModels")
	})
}

func TestCMS_ListCMSItems_ChecksOwningWorkspaceAndDeniesFailClosed(t *testing.T) {
	wsA := accountsid.NewWorkspaceID()

	t.Run("allowed", func(t *testing.T) {
		gw := &fakeCMSGateway{projects: map[string]*cms.Project{"p1": {ID: "p1", WorkspaceID: wsA.String()}}}
		rc := &recordingChecker{allow: true}
		i := newCMSFixture(gw, rc)

		out, err := i.ListCMSItems(context.Background(), "p1", "m1", nil, nil, nil)

		require.NoError(t, err)
		require.NotNil(t, out)
		assert.Equal(t, rbac.ResourceCMSItem, rc.gotResource)
		require.Len(t, rc.gotWorkspace, 1)
		assert.Equal(t, wsA, rc.gotWorkspace[0])
		assert.Equal(t, 1, gw.listItemsCalls)
	})

	t.Run("denied", func(t *testing.T) {
		gw := &fakeCMSGateway{projects: map[string]*cms.Project{"p1": {ID: "p1", WorkspaceID: wsA.String()}}}
		rc := &recordingChecker{allow: false}
		i := newCMSFixture(gw, rc)

		out, err := i.ListCMSItems(context.Background(), "p1", "m1", nil, nil, nil)

		assert.Nil(t, out)
		assert.ErrorIs(t, err, rerror.ErrNotFound)
		assert.Zero(t, gw.listItemsCalls, "denied call must not reach ListItems")
	})
}

func TestCMS_GetCMSModelExportURL_ChecksOwningWorkspaceAndDeniesFailClosed(t *testing.T) {
	wsA := accountsid.NewWorkspaceID()
	geoJSON := cms.ExportTypeGeoJSON

	t.Run("allowed geojson default", func(t *testing.T) {
		gw := &fakeCMSGateway{projects: map[string]*cms.Project{"p1": {ID: "p1", WorkspaceID: wsA.String()}}}
		rc := &recordingChecker{allow: true}
		i := newCMSFixture(gw, rc)

		url, err := i.GetCMSModelExportURL(context.Background(), "p1", "m1", nil)

		require.NoError(t, err)
		assert.NotEmpty(t, url)
		assert.Equal(t, rbac.ResourceCMSModel, rc.gotResource)
		require.Len(t, rc.gotWorkspace, 1)
		assert.Equal(t, wsA, rc.gotWorkspace[0])
		assert.Equal(t, 1, gw.getModelGeoJSONExportURLCalls)
	})

	t.Run("allowed explicit export type", func(t *testing.T) {
		gw := &fakeCMSGateway{projects: map[string]*cms.Project{"p1": {ID: "p1", WorkspaceID: wsA.String()}}}
		rc := &recordingChecker{allow: true}
		i := newCMSFixture(gw, rc)

		url, err := i.GetCMSModelExportURL(context.Background(), "p1", "m1", &geoJSON)

		require.NoError(t, err)
		assert.NotEmpty(t, url)
		assert.Equal(t, 1, gw.getModelExportURLCalls)
	})

	t.Run("denied", func(t *testing.T) {
		gw := &fakeCMSGateway{projects: map[string]*cms.Project{"p1": {ID: "p1", WorkspaceID: wsA.String()}}}
		rc := &recordingChecker{allow: false}
		i := newCMSFixture(gw, rc)

		url, err := i.GetCMSModelExportURL(context.Background(), "p1", "m1", nil)

		assert.Empty(t, url)
		assert.ErrorIs(t, err, rerror.ErrNotFound)
		assert.Zero(t, gw.getModelExportURLCalls)
		assert.Zero(t, gw.getModelGeoJSONExportURLCalls, "denied call must not reach export url gateway methods")
	})
}

func TestCMS_UnresolvableProjectIsDenied(t *testing.T) {
	rc := &recordingChecker{allow: true} // checker would ALLOW
	gw := &fakeCMSGateway{}
	i := newCMSFixture(gw, rc)

	_, err := i.ListCMSAssets(context.Background(), "no-such-project", nil, nil)
	assert.True(t, errors.Is(err, rerror.ErrNotFound) || err != nil, "an unresolvable project must not be allowed through")
	assert.Zero(t, gw.listAssetsCalls)
}
