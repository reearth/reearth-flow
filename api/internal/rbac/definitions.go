package rbac

import (
	"github.com/reearth/reearthx/cerbos/generator"
)

const (
	ServiceName   = "flow"
	PolicyFileDir = "policies"
)

const (
	ResourceUser      = "user"
	ResourceWorkspace = "workspace"
	ResourceProject   = "project"
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
	return builder.
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
		Build()
}
