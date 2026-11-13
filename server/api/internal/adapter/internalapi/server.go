// Package internalapi serves Flow's gRPC internal API (reearth.flow.v1),
// which reearth-dashboard calls. The token check and the caller's identity
// are handled by the interceptors in internal/app; every handler here runs
// through the same usecases as GraphQL.
package internalapi

import (
	"context"
	"errors"
	"fmt"
	"slices"
	"strconv"
	"sync"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/deployment"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	flowv1 "github.com/reearth/reearth-proto/gen/go/flow/v1"
	"github.com/reearth/reearthx/log"
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
	did, err := id.DeploymentIDFrom(req.GetDeploymentId())
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
	return &flowv1.GetDeploymentResponse{Deployment: toDeployment(ds[0])}, nil
}

// Triggers

func (s *Server) ListTriggers(ctx context.Context, req *flowv1.ListTriggersRequest) (*flowv1.ListTriggersResponse, error) {
	wid, err := parseWorkspaceID(req.GetWorkspaceId())
	if err != nil {
		return nil, err
	}
	filter := interfaces.TriggerFilter{Keyword: req.Keyword}
	if req.EventSource != nil {
		es, ok := fromEventSource(req.GetEventSource())
		if !ok {
			return nil, invalidArgument("unknown event_source")
		}
		filter.EventSource = &es
	}
	pagination, page, size := pageParam(req.GetPage())
	ts, info, err := adapter.Usecases(ctx).Trigger.FindByWorkspace(ctx, wid, pagination, filter)
	if err != nil {
		return nil, toStatus(ctx, err)
	}
	triggers, err := triggersWithDeployments(ctx, ts)
	if err != nil {
		return nil, toStatus(ctx, err)
	}
	return &flowv1.ListTriggersResponse{Triggers: triggers, PageInfo: toPageInfo(info, page, size)}, nil
}

func (s *Server) GetTrigger(ctx context.Context, req *flowv1.GetTriggerRequest) (*flowv1.GetTriggerResponse, error) {
	tid, err := id.TriggerIDFrom(req.GetTriggerId())
	if err != nil {
		return nil, invalidArgument("invalid trigger_id")
	}
	t, err := adapter.Usecases(ctx).Trigger.FindByID(ctx, tid)
	if err != nil {
		return nil, toStatus(ctx, hiddenToNotFound(err))
	}
	if t == nil {
		return nil, toStatus(ctx, rerror.ErrNotFound)
	}
	triggers, err := triggersWithDeployments(ctx, []*trigger.Trigger{t})
	if err != nil {
		return nil, toStatus(ctx, err)
	}
	return &flowv1.GetTriggerResponse{Trigger: triggers[0]}, nil
}

func (s *Server) RunTrigger(ctx context.Context, req *flowv1.RunTriggerRequest) (*flowv1.RunTriggerResponse, error) {
	tid, err := id.TriggerIDFrom(req.GetTriggerId())
	if err != nil {
		return nil, invalidArgument("invalid trigger_id")
	}
	origin := fromJobOrigin(req.GetOrigin())
	if origin == nil {
		return nil, invalidArgument("origin.cms_item is required")
	}
	vars := make(map[string]any, len(req.GetVariables()))
	for k, v := range req.GetVariables() {
		vars[k] = v.AsInterface()
	}
	j, err := adapter.Usecases(ctx).Trigger.ExecuteCMSTrigger(ctx, interfaces.ExecuteCMSTriggerParam{
		TriggerID: tid,
		Variables: vars,
		Origin:    *origin,
	})
	if err != nil {
		return nil, toStatus(ctx, err)
	}
	return &flowv1.RunTriggerResponse{Job: toJob(j)}, nil
}

// triggersWithDeployments adds each trigger's deployment and project name, skipping missing or hidden ones.
func triggersWithDeployments(ctx context.Context, ts []*trigger.Trigger) ([]*flowv1.Trigger, error) {
	uc := adapter.Usecases(ctx)

	var dids []id.DeploymentID
	for _, t := range ts {
		if t != nil && !slices.Contains(dids, t.Deployment()) {
			dids = append(dids, t.Deployment())
		}
	}
	deployments := map[id.DeploymentID]*deployment.Deployment{}
	if len(dids) > 0 {
		ds, err := uc.Deployment.Fetch(ctx, dids)
		if err != nil && !errors.Is(err, interfaces.ErrOperationDenied) {
			return nil, err
		}
		for _, d := range ds {
			if d != nil {
				deployments[d.ID()] = d
			}
		}
	}

	var pids []id.ProjectID
	for _, d := range deployments {
		if p := d.Project(); p != nil && !slices.Contains(pids, *p) {
			pids = append(pids, *p)
		}
	}
	names := map[id.ProjectID]string{}
	if len(pids) > 0 {
		ps, err := uc.Project.Fetch(ctx, pids)
		if err != nil && !errors.Is(err, interfaces.ErrOperationDenied) {
			return nil, err
		}
		for _, p := range ps {
			if p != nil {
				names[p.ID()] = p.Name()
			}
		}
	}

	res := make([]*flowv1.Trigger, 0, len(ts))
	for _, t := range ts {
		if t == nil {
			continue
		}
		td := triggerDeployment{deployment: deployments[t.Deployment()]}
		if d := td.deployment; d != nil && d.Project() != nil {
			if name, ok := names[*d.Project()]; ok {
				td.projectName = &name
			}
		}
		res = append(res, toTrigger(t, td))
	}
	return res, nil
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
	return &flowv1.GetJobResponse{Job: toJob(refreshed(ctx, j))}, nil
}

const (
	maxBatchGetJobs        = 100
	maxConcurrentRefreshes = 8
)

func (s *Server) BatchGetJobs(ctx context.Context, req *flowv1.BatchGetJobsRequest) (*flowv1.BatchGetJobsResponse, error) {
	if len(req.GetJobIds()) > maxBatchGetJobs {
		return nil, invalidArgument(fmt.Sprintf("at most %d job_ids", maxBatchGetJobs))
	}
	ids := make([]id.JobID, 0, len(req.GetJobIds()))
	for _, raw := range req.GetJobIds() {
		jid, err := id.JobIDFrom(raw)
		if err != nil {
			return nil, invalidArgument("invalid job_id " + strconv.Quote(raw))
		}
		ids = append(ids, jid)
	}
	if len(ids) == 0 {
		return &flowv1.BatchGetJobsResponse{}, nil
	}
	js, err := adapter.Usecases(ctx).Job.Fetch(ctx, ids)
	if errors.Is(err, interfaces.ErrOperationDenied) {
		return &flowv1.BatchGetJobsResponse{}, nil
	}
	if err != nil {
		return nil, toStatus(ctx, err)
	}
	js = slices.DeleteFunc(js, func(j *job.Job) bool { return j == nil })
	res := make([]*flowv1.Job, len(js))
	sem := make(chan struct{}, maxConcurrentRefreshes)
	var wg sync.WaitGroup
	for i, j := range js {
		wg.Add(1)
		sem <- struct{}{}
		go func() {
			defer func() {
				<-sem
				wg.Done()
			}()
			res[i] = toJob(refreshed(ctx, j))
		}()
	}
	wg.Wait()
	return &flowv1.BatchGetJobsResponse{Jobs: res}, nil
}

func (s *Server) CancelJob(ctx context.Context, req *flowv1.CancelJobRequest) (*flowv1.CancelJobResponse, error) {
	jid, err := id.JobIDFrom(req.GetJobId())
	if err != nil {
		return nil, invalidArgument("invalid job_id")
	}
	j, err := adapter.Usecases(ctx).Job.Cancel(ctx, jid)
	if err != nil {
		return nil, toStatus(ctx, err)
	}
	return &flowv1.CancelJobResponse{Job: toJob(j)}, nil
}

// refreshed updates an unfinished job from its worker, so polling sees it end even without a monitor.
func refreshed(ctx context.Context, j *job.Job) *job.Job {
	switch j.Status() {
	case job.StatusCompleted, job.StatusFailed, job.StatusCancelled:
		return j
	}
	if ctx.Err() != nil {
		return j
	}
	fresh, err := adapter.Usecases(ctx).Job.Refresh(ctx, j)
	if err != nil || fresh == nil {
		log.Warnfc(ctx, "internal API: refreshing job %s: %v", j.ID(), err)
		return j
	}
	return fresh
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
