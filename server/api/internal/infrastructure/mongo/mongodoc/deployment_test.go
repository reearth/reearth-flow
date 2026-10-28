package mongodoc

import (
	"testing"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/deployment"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/parameter"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"go.mongodb.org/mongo-driver/bson"
	"go.mongodb.org/mongo-driver/bson/primitive"
)

func TestDeploymentDocument_KeepsParameters(t *testing.T) {
	pid := id.NewProjectID()
	choice, err := parameter.New().ID(id.NewParameterID()).ProjectID(pid).Name("area").
		Type(parameter.TypeChoice).Index(0).Required(true).Public(true).DefaultValue("tokyo").
		Config(map[string]any{"choices": []any{"tokyo", "osaka"}}).Build()
	require.NoError(t, err)
	d := deployment.New().NewID().Workspace(accountsid.NewWorkspaceID()).Project(&pid).
		WorkflowURL("workflow.json").Version("v1").UpdatedAt(time.Now()).
		Parameters([]*parameter.Parameter{choice}).MustBuild()

	doc, err := NewDeployment(d)
	require.NoError(t, err)
	raw, err := bson.Marshal(doc)
	require.NoError(t, err)
	var decoded DeploymentDocument
	require.NoError(t, bson.Unmarshal(raw, &decoded))
	got, err := decoded.Model()
	require.NoError(t, err)

	require.Len(t, got.Parameters(), 1)
	p := got.Parameters()[0]
	assert.Equal(t, choice.ID(), p.ID())
	assert.Equal(t, "area", p.Name())
	assert.Equal(t, parameter.TypeChoice, p.Type())
	assert.True(t, p.Required())
	assert.True(t, p.Public())
	assert.Equal(t, "tokyo", p.DefaultValue())
	assert.Equal(t, primitive.D{{Key: "choices", Value: primitive.A{"tokyo", "osaka"}}}, p.Config())
}

func TestDeploymentDocument_WithoutParameters(t *testing.T) {
	d := deployment.New().NewID().Workspace(accountsid.NewWorkspaceID()).
		WorkflowURL("workflow.json").Version("v0").UpdatedAt(time.Now()).MustBuild()

	doc, err := NewDeployment(d)
	require.NoError(t, err)
	raw, err := bson.Marshal(doc)
	require.NoError(t, err)
	assert.NotContains(t, bson.Raw(raw).String(), "parameters")
	var decoded DeploymentDocument
	require.NoError(t, bson.Unmarshal(raw, &decoded))
	got, err := decoded.Model()
	require.NoError(t, err)
	assert.Empty(t, got.Parameters())
}

func TestDeploymentDocument_RefusesABrokenParameter(t *testing.T) {
	doc := DeploymentDocument{
		ID:          id.NewDeploymentID().String(),
		WorkspaceID: accountsid.NewWorkspaceID().String(),
		Parameters:  []ParameterDocument{{ID: "not-an-id", Project: id.NewProjectID().String(), Name: "area"}},
	}

	_, err := doc.Model()

	assert.Error(t, err)
}
