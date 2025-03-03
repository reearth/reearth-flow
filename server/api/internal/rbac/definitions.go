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
	ResourceJob           = "job"
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
		AddResource(ResourceJob, []generator.ActionDefinition{
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
			}),
		}).
		AddResource(ResourceUser, []generator.ActionDefinition{
			generator.NewActionDefinition(ActionRead, []string{
				roleSelf,
			}),
			generator.NewActionDefinition(ActionEdit, []string{
				roleSelf,
			}),
		}).
		AddResource(ResourceWorkspace, []generator.ActionDefinition{
			generator.NewActionDefinition(ActionList, []string{
				roleSelf,
			}),
			generator.NewActionDefinition(ActionCreate, []string{
				roleSelf,
			}),
			generator.NewActionDefinition(ActionEdit, []string{
				roleOwner,
			}),
			generator.NewActionDefinition(ActionDelete, []string{
				roleOwner,
			}),
		}).
		Build()
}
