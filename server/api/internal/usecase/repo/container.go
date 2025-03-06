package repo

import (
	"errors"

	"github.com/reearth/reearth-flow/api/internal/usecase"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/reearth/reearthx/account/accountusecase/accountrepo"
	"github.com/reearth/reearthx/authserver"
	"github.com/reearth/reearthx/usecasex"
)

var ErrOperationDenied = errors.New("operation denied")

type Container struct {
	Asset         Asset
	AuthRequest   authserver.RequestRepo
	Config        Config
	Deployment    Deployment
	Job           Job
	Lock          Lock
	Parameter     Parameter
	Project       Project
	ProjectAccess ProjectAccess
	Transaction   usecasex.Transaction
	Trigger       Trigger
	User          accountrepo.User
	Workflow      Workflow
	Workspace     accountrepo.Workspace
}

func (c *Container) AccountRepos() *accountrepo.Container {
	return &accountrepo.Container{
		Workspace:   c.Workspace,
		User:        c.User,
		Transaction: c.Transaction,
	}
}

func (c *Container) Filtered(workspace WorkspaceFilter) *Container {
	if c == nil {
		return c
	}
	return &Container{
		Asset:         c.Asset.Filtered(workspace),
		AuthRequest:   c.AuthRequest,
		Config:        c.Config,
		Deployment:    c.Deployment.Filtered(workspace),
		Job:           c.Job.Filtered(workspace),
		Lock:          c.Lock,
		Workflow:      c.Workflow.Filtered(workspace),
		Parameter:     c.Parameter,
		Project:       c.Project.Filtered(workspace),
		ProjectAccess: c.ProjectAccess,
		Transaction:   c.Transaction,
		Trigger:       c.Trigger,
		User:          c.User,
		Workspace:     c.Workspace,
	}
}

type WorkspaceFilter struct {
	Readable accountdomain.WorkspaceIDList
	Writable accountdomain.WorkspaceIDList
}

func WorkspaceFilterFromOperator(o *usecase.Operator) WorkspaceFilter {
	return WorkspaceFilter{
		Readable: o.AllReadableWorkspaces(),
		Writable: o.AllWritableWorkspaces(),
	}
}

func (f WorkspaceFilter) Clone() WorkspaceFilter {
	return WorkspaceFilter{
		Readable: f.Readable.Clone(),
		Writable: f.Writable.Clone(),
	}
}

func (f WorkspaceFilter) Merge(g WorkspaceFilter) WorkspaceFilter {
	var r, w accountdomain.WorkspaceIDList
	if f.Readable != nil || g.Readable != nil {
		if f.Readable == nil {
			r = g.Readable.Clone()
		} else {
			r = f.Readable.AddUniq(g.Readable...)
		}
	}

	if f.Writable != nil || g.Writable != nil {
		if f.Writable == nil {
			w = g.Writable.Clone()
		} else {
			w = f.Writable.AddUniq(g.Writable...)
		}
	}

	return WorkspaceFilter{
		Readable: r,
		Writable: w,
	}
}

func (f WorkspaceFilter) CanRead(id accountdomain.WorkspaceID) bool {
	return f.Readable == nil || f.Readable.Has(id)
}

func (f WorkspaceFilter) CanWrite(id accountdomain.WorkspaceID) bool {
	return f.Writable == nil || f.Writable.Has(id)
}
