package rbac

import (
	"github.com/reearth/reearthx/cerbos/generator"
)

const (
	ResourceProject  = "project"
	ResourceWorkflow = "workflow"
)

const (
	ActionRead = "read"
	ActionEdit = "edit"
)

const (
	roleOwner      = "owner"
	roleMaintainer = "maintainer"
	roleWriter     = "writer"
	roleReader     = "reader"
)

func DefineResources(builder *generator.ResourceBuilder) []generator.ResourceDefinition {
	return builder.
		AddResource(ResourceProject, []generator.ActionDefinition{
			generator.NewActionDefinition(ActionRead, []string{
				roleOwner,
				roleMaintainer,
				roleWriter,
				roleReader,
			}),
			generator.NewActionDefinition(ActionEdit, []string{
				roleOwner,
				roleMaintainer,
				roleWriter,
			}),
		}).
		AddResource(ResourceWorkflow, []generator.ActionDefinition{
			generator.NewActionDefinition(ActionRead, []string{
				roleOwner,
				roleMaintainer,
				roleWriter,
				roleReader,
			}),
			generator.NewActionDefinition(ActionEdit, []string{
				roleOwner,
				roleMaintainer,
				roleWriter,
			}),
		}).
		Build()
}
