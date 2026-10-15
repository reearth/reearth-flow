// Package internalapi serves Flow's gRPC internal API (reearth.flow.v1),
// which reearth-dashboard calls. The token check and the caller's identity
// are handled by the interceptors in internal/app; every handler here runs
// through the same usecases as GraphQL.
package internalapi

import (
	"context"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/deployment"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	flowv1 "github.com/reearth/reearth-proto/gen/go/flow/v1"
	"github.com/reearth/reearthx/rerror"
)

type Server struct {
	flowv1.UnimplementedReEarthFlowServer
}

var _ flowv1.ReEarthFlowServer = (*Server)(nil)

func NewServer() *Server {
	return &Server{}
}

// Projects

func (s *Server) ListProjects(ctx context.Context, req *flowv1.ListProjectsRequest) (*flowv1.ListProjectsResponse, error) {
	wid, err := parseWorkspaceID(req.GetWorkspaceId())
	if err != nil {
		return nil, err
	}
	param := interfaces.ProjectSearchParam{
		Workspace:       &wid,
		Keyword:         req.GetKeyword(),
		Topics:          req.GetTopics(),
		IncludeArchived: req.GetIncludeArchived(),
	}
	if req.Visibility != nil {
		v, ok := fromVisibility(req.GetVisibility())
		if !ok {
			return nil, invalidArgument("unknown visibility")
		}
		param.Visibility = v
	}
	projects, page, err := searchProjects(ctx, param, req.GetSort(), req.GetPage())
	if err != nil {
		return nil, err
	}
	return &flowv1.ListProjectsResponse{Projects: projects, PageInfo: page}, nil
}

func (s *Server) ListPublicProjects(ctx context.Context, req *flowv1.ListPublicProjectsRequest) (*flowv1.ListPublicProjectsResponse, error) {
	projects, page, err := searchProjects(ctx, interfaces.ProjectSearchParam{
		Keyword: req.GetKeyword(),
		Topics:  req.GetTopics(),
	}, req.GetSort(), req.GetPage())
	if err != nil {
		return nil, err
	}
	return &flowv1.ListPublicProjectsResponse{Projects: projects, PageInfo: page}, nil
}

func searchProjects(ctx context.Context, param interfaces.ProjectSearchParam, sort *flowv1.ProjectSort, page *flowv1.PageRequest) ([]*flowv1.Project, *flowv1.PageInfo, error) {
	switch sort.GetField() {
	case flowv1.ProjectSortField_PROJECT_SORT_FIELD_UNSPECIFIED, flowv1.ProjectSortField_PROJECT_SORT_FIELD_UPDATED_AT:
		param.SortBy = interfaces.ProjectSortUpdatedAt
	case flowv1.ProjectSortField_PROJECT_SORT_FIELD_NAME:
		param.SortBy = interfaces.ProjectSortName
	case flowv1.ProjectSortField_PROJECT_SORT_FIELD_STAR_COUNT:
		param.SortBy = interfaces.ProjectSortStarCount
	default:
		return nil, nil, invalidArgument("unknown sort field")
	}
	param.SortAscending = sort.GetDirection() == flowv1.SortDirection_SORT_DIRECTION_ASC
	param.Page, param.PageSize = interfaces.NormalizePage(int(page.GetPage()), int(page.GetPageSize()))

	projects, info, err := adapter.Usecases(ctx).Project.Search(ctx, param)
	if err != nil {
		return nil, nil, toStatus(ctx, err)
	}
	return toProjects(ctx, projects), toPageInfo(info, param.Page, param.PageSize), nil
}

func (s *Server) GetProject(ctx context.Context, req *flowv1.GetProjectRequest) (*flowv1.GetProjectResponse, error) {
	p, err := findProject(ctx, req.GetProject())
	if err != nil {
		return nil, err
	}
	return &flowv1.GetProjectResponse{Project: toProject(ctx, p)}, nil
}

func (s *Server) CheckProjectAlias(ctx context.Context, req *flowv1.CheckProjectAliasRequest) (*flowv1.CheckProjectAliasResponse, error) {
	wid, err := parseWorkspaceID(req.GetWorkspaceId())
	if err != nil {
		return nil, err
	}
	var pid *id.ProjectID
	if req.ProjectId != nil {
		p, err := id.ProjectIDFrom(req.GetProjectId())
		if err != nil {
			return nil, invalidArgument("invalid project_id")
		}
		pid = &p
	}
	available, err := adapter.Usecases(ctx).Project.CheckAlias(ctx, wid, req.GetAlias(), pid)
	if err != nil {
		return nil, toStatus(ctx, err)
	}
	return &flowv1.CheckProjectAliasResponse{Available: available}, nil
}

func (s *Server) CreateProject(ctx context.Context, req *flowv1.CreateProjectRequest) (*flowv1.CreateProjectResponse, error) {
	wid, err := parseWorkspaceID(req.GetWorkspaceId())
	if err != nil {
		return nil, err
	}
	if req.GetName() == "" {
		return nil, invalidArgument("name is required")
	}
	visibility, ok := fromVisibility(req.GetVisibility())
	if !ok {
		return nil, invalidArgument("unknown visibility")
	}
	name := req.GetName()
	param := interfaces.CreateProjectParam{
		WorkspaceID: wid,
		Name:        &name,
		Description: req.Description,
		Alias:       req.Alias,
		Visibility:  visibility,
		Readme:      req.Readme,
		License:     req.License,
	}
	if len(req.GetTopics()) > 0 {
		param.Topics = req.GetTopics()
	}
	p, err := adapter.Usecases(ctx).Project.Create(ctx, param)
	if err != nil {
		return nil, toStatus(ctx, err)
	}
	return &flowv1.CreateProjectResponse{Project: toProject(ctx, p)}, nil
}

func (s *Server) UpdateProject(ctx context.Context, req *flowv1.UpdateProjectRequest) (*flowv1.UpdateProjectResponse, error) {
	p, err := findProject(ctx, req.GetProject())
	if err != nil {
		return nil, err
	}
	param := interfaces.UpdateProjectParam{
		ID:          p.ID(),
		Name:        req.Name,
		Description: req.Description,
		Archived:    req.Archived,
		Alias:       req.Alias,
		Readme:      req.Readme,
		License:     req.License,
	}
	if req.Name != nil && req.GetName() == "" {
		return nil, invalidArgument("name cannot be empty")
	}
	if req.Visibility != nil {
		v, ok := fromVisibility(req.GetVisibility())
		if !ok || v == nil {
			return nil, invalidArgument("visibility must be public or private")
		}
		param.Visibility = v
	}
	if req.Topics != nil {
		// Non-nil even when empty: an empty list clears the topics.
		param.Topics = append([]string{}, req.GetTopics().GetValues()...)
	}
	updated, err := adapter.Usecases(ctx).Project.Update(ctx, param)
	if err != nil {
		return nil, toStatus(ctx, err)
	}
	return &flowv1.UpdateProjectResponse{Project: toProject(ctx, updated)}, nil
}

func (s *Server) DeleteProject(ctx context.Context, req *flowv1.DeleteProjectRequest) (*flowv1.DeleteProjectResponse, error) {
	p, err := findProject(ctx, req.GetProject())
	if err != nil {
		return nil, err
	}
	if err := adapter.Usecases(ctx).Project.Delete(ctx, p.ID()); err != nil {
		return nil, toStatus(ctx, err)
	}
	return &flowv1.DeleteProjectResponse{ProjectId: p.ID().String()}, nil
}

func (s *Server) SetProjectStar(ctx context.Context, req *flowv1.SetProjectStarRequest) (*flowv1.SetProjectStarResponse, error) {
	p, err := findProject(ctx, req.GetProject())
	if err != nil {
		return nil, err
	}
	updated, err := adapter.Usecases(ctx).Project.SetStar(ctx, p.ID(), req.GetStarred())
	if err != nil {
		return nil, toStatus(ctx, err)
	}
	return &flowv1.SetProjectStarResponse{Project: toProject(ctx, updated)}, nil
}

// findProject returns the project the caller may see, and NOT_FOUND for one
// they may not, before any other check runs. Every project RPC goes through
// it, so none of them reveals whether a hidden project exists.
func findProject(ctx context.Context, ref *flowv1.ProjectRef) (*project.Project, error) {
	uc := adapter.Usecases(ctx).Project
	var p *project.Project
	var err error
	switch r := ref.GetRef().(type) {
	case *flowv1.ProjectRef_Id:
		pid, perr := id.ProjectIDFrom(r.Id)
		if perr != nil {
			return nil, invalidArgument("invalid project id")
		}
		p, err = uc.FindViewable(ctx, pid)
	case *flowv1.ProjectRef_Alias:
		wid, perr := parseWorkspaceID(r.Alias.GetWorkspaceId())
		if perr != nil {
			return nil, perr
		}
		if r.Alias.GetAlias() == "" {
			return nil, invalidArgument("alias is required")
		}
		p, err = uc.FindViewableByAlias(ctx, wid, r.Alias.GetAlias())
	default:
		return nil, invalidArgument("project is required")
	}
	if err != nil {
		return nil, toStatus(ctx, err)
	}
	return p, nil
}

// Deployments

func (s *Server) ListDeployments(ctx context.Context, req *flowv1.ListDeploymentsRequest) (*flowv1.ListDeploymentsResponse, error) {
	wid, err := parseWorkspaceID(req.GetWorkspaceId())
	if err != nil {
		return nil, err
	}
	pagination, page, size := pageParam(req.GetPage())
	ds, info, err := adapter.Usecases(ctx).Deployment.FindByWorkspace(ctx, wid, pagination, req.Keyword)
	if err != nil {
		return nil, toStatus(ctx, err)
	}
	return &flowv1.ListDeploymentsResponse{Deployments: toDeployments(ds), PageInfo: toPageInfo(info, page, size)}, nil
}

func (s *Server) GetDeployment(ctx context.Context, req *flowv1.GetDeploymentRequest) (*flowv1.GetDeploymentResponse, error) {
	d, err := findDeployment(ctx, req.GetDeploymentId())
	if err != nil {
		return nil, err
	}
	return &flowv1.GetDeploymentResponse{Deployment: toDeployment(d)}, nil
}

func (s *Server) DeleteDeployment(ctx context.Context, req *flowv1.DeleteDeploymentRequest) (*flowv1.DeleteDeploymentResponse, error) {
	d, err := findDeployment(ctx, req.GetDeploymentId())
	if err != nil {
		return nil, err
	}
	if err := adapter.Usecases(ctx).Deployment.Delete(ctx, d.ID()); err != nil {
		return nil, toStatus(ctx, err)
	}
	return &flowv1.DeleteDeploymentResponse{DeploymentId: d.ID().String()}, nil
}

func findDeployment(ctx context.Context, deploymentID string) (*deployment.Deployment, error) {
	did, err := id.DeploymentIDFrom(deploymentID)
	if err != nil {
		return nil, invalidArgument("invalid deployment_id")
	}
	ds, err := adapter.Usecases(ctx).Deployment.Fetch(ctx, []id.DeploymentID{did})
	if err != nil {
		return nil, toStatus(ctx, hiddenToNotFound(err))
	}
	if len(ds) == 0 || ds[0] == nil {
		return nil, toStatus(ctx, rerror.ErrNotFound)
	}
	return ds[0], nil
}

// Triggers

func (s *Server) ListTriggers(ctx context.Context, req *flowv1.ListTriggersRequest) (*flowv1.ListTriggersResponse, error) {
	wid, err := parseWorkspaceID(req.GetWorkspaceId())
	if err != nil {
		return nil, err
	}
	pagination, page, size := pageParam(req.GetPage())
	ts, info, err := adapter.Usecases(ctx).Trigger.FindByWorkspace(ctx, wid, pagination, req.Keyword)
	if err != nil {
		return nil, toStatus(ctx, err)
	}
	return &flowv1.ListTriggersResponse{Triggers: toTriggers(ts), PageInfo: toPageInfo(info, page, size)}, nil
}

func (s *Server) GetTrigger(ctx context.Context, req *flowv1.GetTriggerRequest) (*flowv1.GetTriggerResponse, error) {
	t, err := findTrigger(ctx, req.GetTriggerId())
	if err != nil {
		return nil, err
	}
	return &flowv1.GetTriggerResponse{Trigger: toTrigger(t)}, nil
}

func (s *Server) DeleteTrigger(ctx context.Context, req *flowv1.DeleteTriggerRequest) (*flowv1.DeleteTriggerResponse, error) {
	t, err := findTrigger(ctx, req.GetTriggerId())
	if err != nil {
		return nil, err
	}
	if err := adapter.Usecases(ctx).Trigger.Delete(ctx, t.ID()); err != nil {
		return nil, toStatus(ctx, err)
	}
	return &flowv1.DeleteTriggerResponse{TriggerId: t.ID().String()}, nil
}

func findTrigger(ctx context.Context, triggerID string) (*trigger.Trigger, error) {
	tid, err := id.TriggerIDFrom(triggerID)
	if err != nil {
		return nil, invalidArgument("invalid trigger_id")
	}
	t, err := adapter.Usecases(ctx).Trigger.FindByID(ctx, tid)
	if err != nil {
		return nil, toStatus(ctx, hiddenToNotFound(err))
	}
	return t, nil
}

// Jobs

func (s *Server) ListJobs(ctx context.Context, req *flowv1.ListJobsRequest) (*flowv1.ListJobsResponse, error) {
	wid, err := parseWorkspaceID(req.GetWorkspaceId())
	if err != nil {
		return nil, err
	}
	pagination, page, size := pageParam(req.GetPage())
	js, info, err := adapter.Usecases(ctx).Job.FindByWorkspace(ctx, wid, pagination, req.Keyword)
	if err != nil {
		return nil, toStatus(ctx, err)
	}
	return &flowv1.ListJobsResponse{Jobs: toJobs(js), PageInfo: toPageInfo(info, page, size)}, nil
}

func (s *Server) GetJob(ctx context.Context, req *flowv1.GetJobRequest) (*flowv1.GetJobResponse, error) {
	jid, err := id.JobIDFrom(req.GetJobId())
	if err != nil {
		return nil, invalidArgument("invalid job_id")
	}
	j, err := adapter.Usecases(ctx).Job.FindByID(ctx, jid)
	if err != nil {
		return nil, toStatus(ctx, hiddenToNotFound(err))
	}
	return &flowv1.GetJobResponse{Job: toJob(j)}, nil
}

func parseWorkspaceID(s string) (accountsid.WorkspaceID, error) {
	wid, err := accountsid.WorkspaceIDFrom(s)
	if err != nil {
		return wid, invalidArgument("invalid workspace_id")
	}
	return wid, nil
}

func pageParam(p *flowv1.PageRequest) (*interfaces.PaginationParam, int, int) {
	page, size := interfaces.NormalizePage(int(p.GetPage()), int(p.GetPageSize()))
	return &interfaces.PaginationParam{
		Page: &interfaces.PageBasedPaginationParam{Page: page, PageSize: size},
	}, page, size
}
