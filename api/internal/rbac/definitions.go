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
	ResourceMember    = "member"
	ResourceProject   = "project"
)

const (
	ActionList      = "list"
	ActionCreate    = "create"
	ActionEdit      = "edit"
	ActionDelete    = "delete"
	ActionGet       = "get"
	ActionImport    = "import"
	ActionExport    = "export"
	ActionDuplicate = "duplicate"
	ActionTransfer  = "transfer"
)

const (
	roleGeneral = "general"
	roleEditor  = "editor"
	roleAdmin   = "admin"
)

func DefineResources(builder *generator.ResourceBuilder) []generator.ResourceDefinition {
	return builder.
		AddResource(ResourceUser, []generator.ActionDefinition{
			generator.NewActionDefinition(ActionEdit, []string{
				roleGeneral,
			}),
		}).
		AddResource(ResourceWorkspace, []generator.ActionDefinition{
			generator.NewActionDefinition(ActionList, []string{
				roleGeneral,
			}),
			generator.NewActionDefinition(ActionCreate, []string{
				roleGeneral,
			}),
			generator.NewActionDefinition(ActionEdit, []string{
				roleGeneral,
			}),
			generator.NewActionDefinition(ActionDelete, []string{
				roleGeneral,
			}),
		}).
		AddResource(ResourceMember, []generator.ActionDefinition{
			generator.NewActionDefinition(ActionCreate, []string{
				roleAdmin,
			}),
			generator.NewActionDefinition(ActionEdit, []string{
				roleAdmin,
			}),
			generator.NewActionDefinition(ActionDelete, []string{
				roleAdmin,
			}),
		}).
		AddResource(ResourceProject, []generator.ActionDefinition{
			generator.NewActionDefinition(ActionList, []string{
				roleGeneral,
			}),
			generator.NewActionDefinition(ActionCreate, []string{
				roleEditor,
			}),
			generator.NewActionDefinition(ActionEdit, []string{
				roleEditor,
			}),
			generator.NewActionDefinition(ActionDelete, []string{
				roleEditor,
			}),
			generator.NewActionDefinition(ActionGet, []string{
				roleEditor,
			}),
			generator.NewActionDefinition(ActionImport, []string{
				roleEditor,
			}),
			generator.NewActionDefinition(ActionExport, []string{
				roleEditor,
			}),
			generator.NewActionDefinition(ActionDuplicate, []string{
				roleEditor,
			}),
			generator.NewActionDefinition(ActionTransfer, []string{
				roleEditor,
			}),
		}).
		Build()
}
