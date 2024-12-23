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
	ActionList   = "list"
	ActionRead   = "read"
	ActionCreate = "create"
	ActionEdit   = "edit"
	ActionDelete = "delete"
	ActionAny    = "any"
)

const (
	roleSelf       = "self"
	roleReader     = "reader"
	roleEditor     = "editor"
	roleOwner      = "owner"
	roleMaintainer = "maintainer"
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
				roleOwner,
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
			generator.NewActionDefinition(ActionAny, []string{
				roleReader,
				roleEditor,
				roleOwner,
				roleMaintainer,
			}),
		}).
		Build()
}
