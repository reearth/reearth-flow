package rbac

import (
	"github.com/reearth/reearthx/cerbos/generator"
)

const (
	ServiceName   = "flow"
	PolicyFileDir = "policies"
)

const (
	ResourceAsset         = "asset"
	ResourceDeployment    = "deployment"
	ResourceEdge          = "edge"
	ResourceJob           = "job"
	ResourceLog           = "log"
	ResourceNode          = "Node"
	ResourceParameter     = "parameter"
	ResourceProject       = "project"
	ResourceProjectAccess = "projectAccess"
	ResourceTrigger       = "trigger"
	ResourceUser          = "user"
	ResourceWorkspace     = "workspace"
)

const (
	ActionRead   = "read"
	ActionList   = "list"
	ActionCreate = "create"
	ActionEdit   = "edit"
	ActionDelete = "delete"
	ActionAny    = "any"
)

const (
	roleSelf       = "self"
	roleReader     = "reader"
	roleEditor     = "editor"
	roleMaintainer = "maintainer"
	roleOwner      = "owner"
)

func DefineResources(builder *generator.ResourceBuilder) []generator.ResourceDefinition {
	if builder == nil {
		panic("ResourceBuilder cannot be nil")
	}

	return builder.
		AddResource(ResourceAsset, []generator.ActionDefinition{
			generator.NewActionDefinition(ActionAny, []string{
				roleMaintainer,
			}),
		}).
		AddResource(ResourceDeployment, []generator.ActionDefinition{
			generator.NewActionDefinition(ActionAny, []string{
				roleMaintainer,
			}),
		}).
		AddResource(ResourceEdge, []generator.ActionDefinition{
			generator.NewActionDefinition(ActionAny, []string{
				roleMaintainer,
			}),
		}).
		AddResource(ResourceJob, []generator.ActionDefinition{
			generator.NewActionDefinition(ActionAny, []string{
				roleMaintainer,
			}),
		}).
		AddResource(ResourceLog, []generator.ActionDefinition{
			generator.NewActionDefinition(ActionAny, []string{
				roleMaintainer,
			}),
		}).
		AddResource(ResourceNode, []generator.ActionDefinition{
			generator.NewActionDefinition(ActionAny, []string{
				roleMaintainer,
			}),
		}).
		AddResource(ResourceParameter, []generator.ActionDefinition{
			generator.NewActionDefinition(ActionAny, []string{
				roleMaintainer,
			}),
		}).
		AddResource(ResourceProject, []generator.ActionDefinition{
			generator.NewActionDefinition(ActionList, []string{
				roleSelf,
				roleMaintainer,
			}),
			generator.NewActionDefinition(ActionCreate, []string{
				roleMaintainer,
				roleOwner,
			}),
			generator.NewActionDefinition(ActionEdit, []string{
				roleMaintainer,
				roleOwner,
			}),
			generator.NewActionDefinition(ActionDelete, []string{
				roleMaintainer,
				roleOwner,
			}),
			generator.NewActionDefinition(ActionAny, []string{
				roleReader,
				roleEditor,
				roleOwner,
				roleMaintainer,
			}),
		}).
		AddResource(ResourceProjectAccess, []generator.ActionDefinition{
			generator.NewActionDefinition(ActionAny, []string{
				roleMaintainer,
			}),
		}).
		AddResource(ResourceTrigger, []generator.ActionDefinition{
			generator.NewActionDefinition(ActionCreate, []string{
				roleMaintainer,
				roleOwner,
			}),
			generator.NewActionDefinition(ActionEdit, []string{
				roleMaintainer,
				roleOwner,
			}),
			generator.NewActionDefinition(ActionDelete, []string{
				roleMaintainer,
				roleOwner,
			}),
			generator.NewActionDefinition(ActionAny, []string{
				roleSelf,
				roleMaintainer,
			}),
		}).
		AddResource(ResourceUser, []generator.ActionDefinition{
			generator.NewActionDefinition(ActionRead, []string{
				roleSelf,
				roleMaintainer,
			}),
			generator.NewActionDefinition(ActionEdit, []string{
				roleSelf,
				roleMaintainer,
			}),
		}).
		AddResource(ResourceWorkspace, []generator.ActionDefinition{
			generator.NewActionDefinition(ActionList, []string{
				roleSelf,
				roleMaintainer,
			}),
			generator.NewActionDefinition(ActionCreate, []string{
				roleSelf,
				roleMaintainer,
			}),
			generator.NewActionDefinition(ActionEdit, []string{
				roleOwner,
				roleMaintainer,
			}),
			generator.NewActionDefinition(ActionDelete, []string{
				roleOwner,
				roleMaintainer,
			}),
		}).
		Build()
}
