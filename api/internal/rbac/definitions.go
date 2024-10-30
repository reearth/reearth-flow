package rbac

const (
	serviceName = "flow"
)

type resourceType string

const (
	resourceProject  resourceType = "project"
	resourceWorkflow resourceType = "workflow"
)

type actionType string

const (
	actionRead actionType = "read"
	actionEdit actionType = "edit"
)

type roleType string

const (
	roleOwner      roleType = "owner"
	roleMaintainer roleType = "maintainer"
	roleWriter     roleType = "writer"
	roleReader     roleType = "reader"
)

type resourceDefinition struct {
	resource string
	actions  []actionDefinition
}

type actionDefinition struct {
	action actionType
	roles  []roleType
}

func makeResourceName(resource resourceType) string {
	return serviceName + ":" + string(resource)
}

func DefineResources() []resourceDefinition {
	return []resourceDefinition{
		{
			resource: makeResourceName(resourceProject),
			actions: []actionDefinition{
				{
					action: actionRead,
					roles:  []roleType{roleOwner, roleMaintainer, roleWriter, roleReader},
				},
				{
					action: actionEdit,
					roles:  []roleType{roleOwner, roleMaintainer, roleWriter},
				},
			},
		},
		{
			resource: makeResourceName(resourceWorkflow),
			actions: []actionDefinition{
				{
					action: actionRead,
					roles:  []roleType{roleOwner, roleMaintainer, roleWriter, roleReader},
				},
				{
					action: actionEdit,
					roles:  []roleType{roleOwner, roleMaintainer, roleWriter},
				},
			},
		},
	}
}
