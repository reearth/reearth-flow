package repo

import (
	"errors"

	"github.com/reearth/reearth-flow/api/pkg/id"
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
	EdgeExecution EdgeExecution
	Job           Job
	Lock          Lock
	NodeExecution NodeExecution
	Parameter     Parameter
	Permittable   accountrepo.Permittable // TODO: Delete this once the permission check migration is complete.
	Project       Project
	ProjectAccess ProjectAccess
	Role          accountrepo.Role // TODO: Delete this once the permission check migration is complete.
	Transaction   usecasex.Transaction
	Trigger       Trigger
	User          accountrepo.User // TODO: Remove this once the replace user management is complete.
	WorkerConfig  WorkerConfig
	Workflow      Workflow
	Workspace     accountrepo.Workspace // TODO: Remove this once the replace user management is complete.
}

// TODO: Remove this once the replace user management is complete.
func (c *Container) AccountRepos() *accountrepo.Container {
	return &accountrepo.Container{
		Workspace:   c.Workspace,
		User:        c.User,
		Transaction: c.Transaction,
		Role:        c.Role,
		Permittable: c.Permittable,
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
		EdgeExecution: c.EdgeExecution,
		Job:           c.Job.Filtered(workspace),
		Lock:          c.Lock,
		NodeExecution: c.NodeExecution,
		Parameter:     c.Parameter,
		Project:       c.Project.Filtered(workspace),
		ProjectAccess: c.ProjectAccess,
		Transaction:   c.Transaction,
		Trigger:       c.Trigger,
		User:          c.User,
		Workflow:      c.Workflow,
		Workspace:     c.Workspace,
	}
}

type WorkspaceFilter struct {
	Readable id.WorkspaceIDList
	Writable id.WorkspaceIDList
}

func (f WorkspaceFilter) Clone() WorkspaceFilter {
	return WorkspaceFilter{
		Readable: f.Readable.Clone(),
		Writable: f.Writable.Clone(),
	}
}

func (f WorkspaceFilter) Merge(g WorkspaceFilter) WorkspaceFilter {
	var r, w id.WorkspaceIDList
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

func (f WorkspaceFilter) CanRead(id id.WorkspaceID) bool {
	return f.Readable == nil || f.Readable.Has(id)
}

func (f WorkspaceFilter) CanWrite(id id.WorkspaceID) bool {
	return f.Writable == nil || f.Writable.Has(id)
}
