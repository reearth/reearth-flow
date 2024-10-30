package rbac

import (
	"github.com/reearth/reearthx/cerbos/generator"
)

const (
	serviceName = "flow"
)

const (
	resourceProject  = "project"
	resourceWorkflow = "workflow"
)

const (
	actionRead = "read"
	actionEdit = "edit"
)

const (
	roleOwner      = "owner"
	roleMaintainer = "maintainer"
	roleWriter     = "writer"
	roleReader     = "reader"
)

type ResourceDefinition struct {
	Resource string
	Actions  []ActionDefinition
}

type ActionDefinition struct {
	Action string
	Roles  []string
}

func makeResourceName(resource string) string {
	return serviceName + ":" + resource
}

func DefineResources() []ResourceDefinition {
	return []ResourceDefinition{
		{
			Resource: makeResourceName(resourceProject),
			Actions: []ActionDefinition{
				{
					Action: actionRead,
					Roles:  []string{roleOwner, roleMaintainer, roleWriter, roleReader},
				},
				{
					Action: actionEdit,
					Roles:  []string{roleOwner, roleMaintainer, roleWriter},
				},
			},
		},
		{
			Resource: makeResourceName(resourceWorkflow),
			Actions: []ActionDefinition{
				{
					Action: actionRead,
					Roles:  []string{roleOwner, roleMaintainer, roleWriter, roleReader},
				},
				{
					Action: actionEdit,
					Roles:  []string{roleOwner, roleMaintainer, roleWriter},
				},
			},
		},
	}
}

func (r ResourceDefinition) GetResource() string {
	return r.Resource
}

func (r ResourceDefinition) GetActions() []generator.ActionDefinition {
	actions := make([]generator.ActionDefinition, len(r.Actions))
	for i, a := range r.Actions {
		actions[i] = a
	}
	return actions
}

func (a ActionDefinition) GetAction() string {
	return a.Action
}

func (a ActionDefinition) GetRoles() []string {
	return a.Roles
}
