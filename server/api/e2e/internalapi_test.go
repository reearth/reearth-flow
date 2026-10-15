package e2e

import (
	"context"
	"net"
	"testing"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/app"
	"github.com/reearth/reearth-flow/api/internal/app/config"
	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/deployment"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	flowv1 "github.com/reearth/reearth-proto/gen/go/flow/v1"
	"github.com/reearth/reearthx/account/accountdomain/user"
	"github.com/reearth/reearthx/account/accountdomain/workspace"
	"github.com/samber/lo"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"
)

const internalAPIToken = "internal-token"

// denyPrivate is a plan policy that allows private projects in paid only.
type denyPrivate struct{ paid accountsid.WorkspaceID }

func (d denyPrivate) CheckPolicy(_ context.Context, req gateway.PolicyCheckRequest) (*gateway.PolicyCheckResponse, error) {
	return &gateway.PolicyCheckResponse{CheckType: req.CheckType, Allowed: req.WorkspaceID == d.paid}, nil
}

type internalAPIFixture struct {
	client                            flowv1.ReEarthFlowClient
	repos                             *repo.Container
	owner, writer, reader, outsider   *user.User
	ws, free                          *workspace.Workspace
	publicProject, privateProject     *project.Project
	deployment, deploymentWithTrigger *deployment.Deployment
	trigger                           *trigger.Trigger
	job                               *job.Job
	wsID, freeID                      string
}

func startInternalAPI(t *testing.T) internalAPIFixture {
	t.Helper()
	if testing.Short() {
		t.Skip("skipping test in short mode.")
	}
	ctx := context.Background()
	repos := initRepos(t, true)
	acRepos := repos.AccountRepos()

	newUser := func(name string) *user.User {
		u := user.New().NewID().Name(name).Email(name + "@e2e.com").Workspace(workspace.NewID()).MustBuild()
		require.NoError(t, acRepos.User.Save(ctx, u))
		return u
	}
	f := internalAPIFixture{
		repos:    repos,
		owner:    newUser("owner"),
		writer:   newUser("writer"),
		reader:   newUser("reader"),
		outsider: newUser("outsider"),
	}
	f.ws = workspace.New().NewID().Name("team").Alias("team").Members(map[user.ID]workspace.Member{
		f.owner.ID():  {Role: workspace.RoleOwner},
		f.writer.ID(): {Role: workspace.RoleWriter},
		f.reader.ID(): {Role: workspace.RoleReader},
	}).MustBuild()
	f.free = workspace.New().NewID().Name("free").Alias("free").Members(map[user.ID]workspace.Member{
		f.owner.ID(): {Role: workspace.RoleOwner},
	}).MustBuild()
	require.NoError(t, acRepos.Workspace.Save(ctx, f.ws))
	require.NoError(t, acRepos.Workspace.Save(ctx, f.free))
	f.wsID, f.freeID = f.ws.ID().String(), f.free.ID().String()
	wid := accountsid.MustWorkspaceID(f.wsID)

	f.publicProject = project.New().NewID().Workspace(wid).Name("public one").Alias("public-one").
		Visibility(project.VisibilityPublic).Topics([]string{"gis"}).MustBuild()
	f.privateProject = project.New().NewID().Workspace(wid).Name("private one").Alias("private-one").
		Visibility(project.VisibilityPrivate).MustBuild()
	require.NoError(t, repos.Project.Save(ctx, f.publicProject))
	require.NoError(t, repos.Project.Save(ctx, f.privateProject))

	f.deployment = deployment.New().NewID().Workspace(wid).Project(f.publicProject.ID().Ref()).
		Version("v1").Description("nightly").WorkflowURL("https://example.com/w.yml").MustBuild()
	f.deploymentWithTrigger = deployment.New().NewID().Workspace(wid).
		Version("v1").Description("scheduled").WorkflowURL("https://example.com/w2.yml").MustBuild()
	require.NoError(t, repos.Deployment.Save(ctx, f.deployment))
	require.NoError(t, repos.Deployment.Save(ctx, f.deploymentWithTrigger))

	f.trigger = trigger.New().NewID().Workspace(wid).Deployment(f.deploymentWithTrigger.ID()).
		Description("every day").EventSource(trigger.EventSourceTypeAPIDriven).AuthToken("secret").MustBuild()
	require.NoError(t, repos.Trigger.Save(ctx, f.trigger))

	f.job = lo.Must(job.New().NewID().Workspace(wid).Deployment(f.deployment.ID().Ref()).
		Status(job.StatusCompleted).StartedAt(time.Now()).Build())
	require.NoError(t, repos.Job.Save(ctx, f.job))

	srv := app.NewInternalAPIServer(&app.ServerConfig{
		Config:       &config.Config{InternalAPI_Token: internalAPIToken},
		Repos:        repos,
		AccountRepos: acRepos,
		Gateways: &gateway.Container{
			File:          initGateway().File,
			PolicyChecker: denyPrivate{paid: wid},
		},
	})
	l, err := net.Listen("tcp", "127.0.0.1:0")
	require.NoError(t, err)
	go func() { _ = srv.Serve(l) }()
	t.Cleanup(srv.Stop)

	conn, err := grpc.NewClient(l.Addr().String(), grpc.WithTransportCredentials(insecure.NewCredentials()))
	require.NoError(t, err)
	t.Cleanup(func() { _ = conn.Close() })
	f.client = flowv1.NewReEarthFlowClient(conn)
	return f
}

// as returns a context carrying the token, and the user's ID when u is set.
func as(u *user.User) context.Context {
	md := metadata.Pairs("authorization", "Bearer "+internalAPIToken)
	if u != nil {
		md.Set("user-id", u.ID().String())
	}
	return metadata.NewOutgoingContext(context.Background(), md)
}

func assertCode(t *testing.T, err error, code codes.Code, msgAndArgs ...any) {
	t.Helper()
	require.Error(t, err, msgAndArgs...)
	assert.Equal(t, code, status.Code(err), append([]any{"error: %v", err}, msgAndArgs...)...)
}

func projectRef(p *project.Project) *flowv1.ProjectRef {
	return &flowv1.ProjectRef{Ref: &flowv1.ProjectRef_Id{Id: p.ID().String()}}
}

func projectNames(ps []*flowv1.Project) []string {
	return lo.Map(ps, func(p *flowv1.Project, _ int) string { return p.GetName() })
}

func TestInternalAPI_Auth(t *testing.T) {
	f := startInternalAPI(t)
	list := &flowv1.ListProjectsRequest{WorkspaceId: f.wsID}

	_, err := f.client.ListProjects(context.Background(), list)
	assertCode(t, err, codes.Unauthenticated)

	wrong := metadata.NewOutgoingContext(context.Background(), metadata.Pairs("authorization", "Bearer nope", "user-id", f.owner.ID().String()))
	_, err = f.client.ListProjects(wrong, list)
	assertCode(t, err, codes.Unauthenticated)

	_, err = f.client.ListDeployments(as(nil), &flowv1.ListDeploymentsRequest{WorkspaceId: f.wsID})
	assertCode(t, err, codes.Unauthenticated)

	unknown := metadata.NewOutgoingContext(context.Background(), metadata.Pairs("authorization", "Bearer "+internalAPIToken, "user-id", accountsid.NewUserID().String()))
	_, err = f.client.ListProjects(unknown, list)
	assertCode(t, err, codes.Unauthenticated)

	_, err = f.client.ListProjects(as(nil), list)
	assert.NoError(t, err, "listing public projects needs no user")
}

func TestInternalAPI_ProjectsAsMember(t *testing.T) {
	f := startInternalAPI(t)
	ctx := as(f.owner)

	res, err := f.client.ListProjects(ctx, &flowv1.ListProjectsRequest{
		WorkspaceId: f.wsID,
		Sort:        &flowv1.ProjectSort{Field: flowv1.ProjectSortField_PROJECT_SORT_FIELD_NAME, Direction: flowv1.SortDirection_SORT_DIRECTION_ASC},
	})
	require.NoError(t, err)
	assert.Equal(t, []string{"private one", "public one"}, projectNames(res.GetProjects()))
	assert.Equal(t, int64(2), res.GetPageInfo().GetTotalCount())
	assert.Equal(t, int32(20), res.GetPageInfo().GetPageSize(), "the default page size")

	created, err := f.client.CreateProject(ctx, &flowv1.CreateProjectRequest{
		WorkspaceId: f.wsID,
		Name:        "made here",
		Alias:       lo.ToPtr("made-here"),
		Visibility:  flowv1.ProjectVisibility_PROJECT_VISIBILITY_PRIVATE,
		Readme:      lo.ToPtr("# Hi"),
		Topics:      []string{"gis", " gis "},
	})
	require.NoError(t, err)
	p := created.GetProject()
	assert.Equal(t, "made-here", p.GetAlias())
	assert.Equal(t, flowv1.ProjectVisibility_PROJECT_VISIBILITY_PRIVATE, p.GetVisibility())
	assert.Equal(t, []string{"gis"}, p.GetTopics())

	_, err = f.client.CreateProject(ctx, &flowv1.CreateProjectRequest{WorkspaceId: f.wsID, Name: "dup", Alias: lo.ToPtr("made-here")})
	assertCode(t, err, codes.AlreadyExists)
	_, err = f.client.CreateProject(ctx, &flowv1.CreateProjectRequest{WorkspaceId: f.wsID, Name: "bad", Alias: lo.ToPtr("p-mine")})
	assertCode(t, err, codes.InvalidArgument)
	_, err = f.client.CreateProject(ctx, &flowv1.CreateProjectRequest{WorkspaceId: f.wsID})
	assertCode(t, err, codes.InvalidArgument)

	byAlias := &flowv1.ProjectRef{Ref: &flowv1.ProjectRef_Alias{Alias: &flowv1.ProjectAlias{WorkspaceId: f.wsID, Alias: "made-here"}}}
	got, err := f.client.GetProject(ctx, &flowv1.GetProjectRequest{Project: byAlias})
	require.NoError(t, err)
	assert.Equal(t, p.GetId(), got.GetProject().GetId())

	alias, err := f.client.CheckProjectAlias(ctx, &flowv1.CheckProjectAliasRequest{WorkspaceId: f.wsID, Alias: "made-here"})
	require.NoError(t, err)
	assert.False(t, alias.GetAvailable())
	alias, err = f.client.CheckProjectAlias(ctx, &flowv1.CheckProjectAliasRequest{WorkspaceId: f.wsID, Alias: "made-here", ProjectId: lo.ToPtr(p.GetId())})
	require.NoError(t, err)
	assert.True(t, alias.GetAvailable())

	updated, err := f.client.UpdateProject(ctx, &flowv1.UpdateProjectRequest{
		Project:    byAlias,
		Name:       lo.ToPtr("renamed"),
		Visibility: lo.ToPtr(flowv1.ProjectVisibility_PROJECT_VISIBILITY_PUBLIC),
		Topics:     &flowv1.Topics{},
	})
	require.NoError(t, err)
	assert.Equal(t, "renamed", updated.GetProject().GetName())
	assert.Equal(t, flowv1.ProjectVisibility_PROJECT_VISIBILITY_PUBLIC, updated.GetProject().GetVisibility())
	assert.Empty(t, updated.GetProject().GetTopics(), "an empty Topics clears them")
	assert.Equal(t, "# Hi", updated.GetProject().GetReadme(), "fields left unset are kept")

	starred, err := f.client.SetProjectStar(ctx, &flowv1.SetProjectStarRequest{Project: byAlias, Starred: true})
	require.NoError(t, err)
	assert.True(t, starred.GetProject().GetStarred())
	assert.Equal(t, int64(1), starred.GetProject().GetStarCount())

	deleted, err := f.client.DeleteProject(ctx, &flowv1.DeleteProjectRequest{Project: byAlias})
	require.NoError(t, err)
	assert.Equal(t, p.GetId(), deleted.GetProjectId())
	_, err = f.client.GetProject(ctx, &flowv1.GetProjectRequest{Project: byAlias})
	assertCode(t, err, codes.NotFound)
}

func TestInternalAPI_PrivateNeedsPlan(t *testing.T) {
	f := startInternalAPI(t)
	ctx := as(f.owner)

	_, err := f.client.CreateProject(ctx, &flowv1.CreateProjectRequest{
		WorkspaceId: f.freeID, Name: "secret", Visibility: flowv1.ProjectVisibility_PROJECT_VISIBILITY_PRIVATE,
	})
	assertCode(t, err, codes.FailedPrecondition)

	res, err := f.client.CreateProject(ctx, &flowv1.CreateProjectRequest{WorkspaceId: f.freeID, Name: "open"})
	require.NoError(t, err)
	assert.Equal(t, flowv1.ProjectVisibility_PROJECT_VISIBILITY_PUBLIC, res.GetProject().GetVisibility(), "public by default")
}

func TestInternalAPI_ProjectsAsReader(t *testing.T) {
	f := startInternalAPI(t)
	ctx := as(f.reader)

	res, err := f.client.ListProjects(ctx, &flowv1.ListProjectsRequest{WorkspaceId: f.wsID})
	require.NoError(t, err)
	assert.Len(t, res.GetProjects(), 2, "readers see private projects")

	_, err = f.client.DeleteProject(ctx, &flowv1.DeleteProjectRequest{Project: projectRef(f.privateProject)})
	assertCode(t, err, codes.PermissionDenied)

	_, err = f.client.CreateProject(ctx, &flowv1.CreateProjectRequest{WorkspaceId: f.wsID, Name: "nope"})
	assertCode(t, err, codes.PermissionDenied)
}

func TestInternalAPI_ProjectsAsOutsider(t *testing.T) {
	f := startInternalAPI(t)

	for name, ctx := range map[string]context.Context{"outsider": as(f.outsider), "anonymous": as(nil)} {
		t.Run(name, func(t *testing.T) {
			res, err := f.client.ListProjects(ctx, &flowv1.ListProjectsRequest{WorkspaceId: f.wsID})
			require.NoError(t, err)
			assert.Equal(t, []string{"public one"}, projectNames(res.GetProjects()))

			res, err = f.client.ListProjects(ctx, &flowv1.ListProjectsRequest{
				WorkspaceId: f.wsID,
				Visibility:  lo.ToPtr(flowv1.ProjectVisibility_PROJECT_VISIBILITY_PRIVATE),
			})
			require.NoError(t, err)
			assert.Empty(t, res.GetProjects())

			_, err = f.client.GetProject(ctx, &flowv1.GetProjectRequest{Project: projectRef(f.publicProject)})
			assert.NoError(t, err)
			_, err = f.client.GetProject(ctx, &flowv1.GetProjectRequest{Project: projectRef(f.privateProject)})
			assertCode(t, err, codes.NotFound)

			pub, err := f.client.ListPublicProjects(ctx, &flowv1.ListPublicProjectsRequest{Topics: []string{"gis"}})
			require.NoError(t, err)
			assert.Equal(t, []string{"public one"}, projectNames(pub.GetProjects()))
		})
	}

	ctx := as(f.outsider)
	starred, err := f.client.SetProjectStar(ctx, &flowv1.SetProjectStarRequest{Project: projectRef(f.publicProject), Starred: true})
	require.NoError(t, err, "anyone may star a public project")
	assert.True(t, starred.GetProject().GetStarred())

	_, err = f.client.SetProjectStar(ctx, &flowv1.SetProjectStarRequest{Project: projectRef(f.privateProject), Starred: true})
	assertCode(t, err, codes.NotFound)
	_, err = f.client.UpdateProject(ctx, &flowv1.UpdateProjectRequest{Project: projectRef(f.publicProject), Name: lo.ToPtr("mine now")})
	assertCode(t, err, codes.PermissionDenied)
	_, err = f.client.DeleteProject(ctx, &flowv1.DeleteProjectRequest{Project: projectRef(f.privateProject)})
	assertCode(t, err, codes.NotFound, "a hidden project is not revealed by a delete")

	_, err = f.client.SetProjectStar(as(nil), &flowv1.SetProjectStarRequest{Project: projectRef(f.publicProject), Starred: true})
	assertCode(t, err, codes.Unauthenticated)
}

func TestInternalAPI_DeploymentsTriggersJobs(t *testing.T) {
	f := startInternalAPI(t)

	t.Run("writer", func(t *testing.T) {
		ctx := as(f.writer)
		ds, err := f.client.ListDeployments(ctx, &flowv1.ListDeploymentsRequest{WorkspaceId: f.wsID})
		require.NoError(t, err)
		assert.Len(t, ds.GetDeployments(), 2)

		d, err := f.client.GetDeployment(ctx, &flowv1.GetDeploymentRequest{DeploymentId: f.deployment.ID().String()})
		require.NoError(t, err)
		assert.Equal(t, f.publicProject.ID().String(), d.GetDeployment().GetProjectId())
		assert.Equal(t, "v1", d.GetDeployment().GetVersion())

		trg, err := f.client.GetTrigger(ctx, &flowv1.GetTriggerRequest{TriggerId: f.trigger.ID().String()})
		require.NoError(t, err)
		assert.Equal(t, flowv1.TriggerEventSource_TRIGGER_EVENT_SOURCE_API_DRIVEN, trg.GetTrigger().GetEventSource())

		js, err := f.client.ListJobs(ctx, &flowv1.ListJobsRequest{WorkspaceId: f.wsID})
		require.NoError(t, err)
		require.Len(t, js.GetJobs(), 1)
		assert.Equal(t, flowv1.JobStatus_JOB_STATUS_COMPLETED, js.GetJobs()[0].GetStatus())

		j, err := f.client.GetJob(ctx, &flowv1.GetJobRequest{JobId: f.job.ID().String()})
		require.NoError(t, err)
		assert.Equal(t, f.deployment.ID().String(), j.GetJob().GetDeploymentId())

		_, err = f.client.DeleteDeployment(ctx, &flowv1.DeleteDeploymentRequest{DeploymentId: f.deploymentWithTrigger.ID().String()})
		assertCode(t, err, codes.FailedPrecondition)

		_, err = f.client.DeleteTrigger(ctx, &flowv1.DeleteTriggerRequest{TriggerId: f.trigger.ID().String()})
		assertCode(t, err, codes.PermissionDenied, "deleting a trigger needs maintainer")
	})

	t.Run("reader", func(t *testing.T) {
		ctx := as(f.reader)
		_, err := f.client.ListDeployments(ctx, &flowv1.ListDeploymentsRequest{WorkspaceId: f.wsID})
		assertCode(t, err, codes.PermissionDenied)
		_, err = f.client.GetDeployment(ctx, &flowv1.GetDeploymentRequest{DeploymentId: f.deployment.ID().String()})
		assertCode(t, err, codes.NotFound)
		_, err = f.client.GetJob(ctx, &flowv1.GetJobRequest{JobId: f.job.ID().String()})
		assertCode(t, err, codes.NotFound)

		ts, err := f.client.ListTriggers(ctx, &flowv1.ListTriggersRequest{WorkspaceId: f.wsID})
		require.NoError(t, err, "every member may see triggers")
		assert.Len(t, ts.GetTriggers(), 1)
	})

	t.Run("outsider", func(t *testing.T) {
		ctx := as(f.outsider)
		_, err := f.client.GetDeployment(ctx, &flowv1.GetDeploymentRequest{DeploymentId: f.deployment.ID().String()})
		assertCode(t, err, codes.NotFound)
		_, err = f.client.GetTrigger(ctx, &flowv1.GetTriggerRequest{TriggerId: f.trigger.ID().String()})
		assertCode(t, err, codes.NotFound)
		_, err = f.client.DeleteTrigger(ctx, &flowv1.DeleteTriggerRequest{TriggerId: f.trigger.ID().String()})
		assertCode(t, err, codes.NotFound)
		_, err = f.client.ListTriggers(ctx, &flowv1.ListTriggersRequest{WorkspaceId: f.wsID})
		assertCode(t, err, codes.PermissionDenied)
	})

	t.Run("owner deletes", func(t *testing.T) {
		ctx := as(f.owner)
		_, err := f.client.DeleteTrigger(ctx, &flowv1.DeleteTriggerRequest{TriggerId: f.trigger.ID().String()})
		require.NoError(t, err)
		_, err = f.client.DeleteDeployment(ctx, &flowv1.DeleteDeploymentRequest{DeploymentId: f.deploymentWithTrigger.ID().String()})
		require.NoError(t, err, "no trigger is left")
		_, err = f.client.GetDeployment(ctx, &flowv1.GetDeploymentRequest{DeploymentId: f.deploymentWithTrigger.ID().String()})
		assertCode(t, err, codes.NotFound)

		_, err = f.client.GetJob(ctx, &flowv1.GetJobRequest{JobId: "not-an-id"})
		assertCode(t, err, codes.InvalidArgument)
	})
}

// The API trigger's token must never leave the server.
func TestInternalAPI_TriggerHasNoToken(t *testing.T) {
	f := startInternalAPI(t)
	res, err := f.client.GetTrigger(as(f.owner), &flowv1.GetTriggerRequest{TriggerId: f.trigger.ID().String()})
	require.NoError(t, err)
	assert.NotContains(t, res.String(), "secret")
}
