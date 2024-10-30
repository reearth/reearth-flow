package rbac

import (
	"github.com/reearth/reearthx/cerbos/generator"
)

func DefineResources(builder *generator.ResourceBuilder) []generator.ResourceDefinition {
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

	return builder.
		AddResource(resourceProject, []generator.ActionDefinition{
			generator.NewActionDefinition(actionRead, []string{
				roleOwner,
				roleMaintainer,
				roleWriter,
				roleReader,
			}),
			generator.NewActionDefinition(actionEdit, []string{
				roleOwner,
				roleMaintainer,
				roleWriter,
			}),
		}).
		AddResource(resourceWorkflow, []generator.ActionDefinition{
			generator.NewActionDefinition(actionRead, []string{
				roleOwner,
				roleMaintainer,
				roleWriter,
				roleReader,
			}),
			generator.NewActionDefinition(actionEdit, []string{
				roleOwner,
				roleMaintainer,
				roleWriter,
			}),
		}).
		Build()
}
