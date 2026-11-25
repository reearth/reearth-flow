package rbac

import (
	"github.com/reearth/reearthx/cerbos/generator"
)

const (
	ServiceName   = "flow"
	PolicyFileDir = "policies"
)

const (
	ResourceAsset           = "asset"
	ResourceCMSAsset        = "cms_asset"
	ResourceCMSItem         = "cms_item"
	ResourceCMSModel        = "cms_model"
	ResourceCMSProject      = "cms_project"
	ResourceDeployment      = "deployment"
	ResourceEdge            = "edge"
	ResourceJob             = "job"
	ResourceLog             = "log"
	ResourceNode            = "Node"
	ResourceParameter       = "parameter"
	ResourceProject         = "project"
	ResourceProjectAccess   = "projectAccess"
	ResourceProjectDocument = "projectDocument"
	ResourceTrigger         = "trigger"
	ResourceUser            = "user"
	ResourceWorkspace       = "workspace"
)

const (
	ActionRead          = "read"
	ActionList          = "list"
	ActionCreate        = "create"
	ActionEdit          = "edit"
	ActionDelete        = "delete"
	ActionAny           = "any"
	ActionManageMembers = "manageMembers"
)

const (
	roleSelf       = "self"
	roleReader     = "reader"
	roleWriter     = "writer"
	roleMaintainer = "maintainer"
	roleOwner      = "owner"
)

// DefineResources returns the flow RBAC policy as declarative resource rules.
// reearthx's generator.GeneratePolicies builds Cerbos policies from these, and
// each action matches its own key only: ActionAny is not a wildcard.
func DefineResources() []generator.ResourceRule {
	allRoles := []string{roleReader, roleWriter, roleMaintainer, roleOwner}
	writerUp := []string{roleWriter, roleMaintainer, roleOwner}
	maintainerUp := []string{roleMaintainer, roleOwner}

	readAllWrite := func(writers []string) map[string]generator.ActionRule {
		return map[string]generator.ActionRule{
			ActionRead: {Roles: allRoles},
			ActionAny:  {Roles: writers},
		}
	}
	anyAction := func(roles []string) map[string]generator.ActionRule {
		return map[string]generator.ActionRule{ActionAny: {Roles: roles}}
	}

	return []generator.ResourceRule{
		{Resource: ResourceAsset, Actions: readAllWrite(writerUp)},
		{Resource: ResourceCMSAsset, Actions: anyAction(writerUp)},
		{Resource: ResourceCMSItem, Actions: anyAction(writerUp)},
		{Resource: ResourceCMSModel, Actions: anyAction(writerUp)},
		{Resource: ResourceCMSProject, Actions: anyAction(writerUp)},
		{Resource: ResourceDeployment, Actions: readAllWrite(maintainerUp)},
		{Resource: ResourceEdge, Actions: readAllWrite(writerUp)},
		{Resource: ResourceJob, Actions: readAllWrite(writerUp)},
		{Resource: ResourceLog, Actions: map[string]generator.ActionRule{
			ActionRead: {Roles: allRoles},
		}},
		{Resource: ResourceNode, Actions: anyAction(writerUp)},
		{Resource: ResourceParameter, Actions: readAllWrite(writerUp)},
		{Resource: ResourceProject, Actions: map[string]generator.ActionRule{
			ActionList:   {Roles: []string{roleSelf}},
			ActionRead:   {Roles: allRoles},
			ActionCreate: {Roles: maintainerUp},
			ActionEdit:   {Roles: maintainerUp},
			ActionDelete: {Roles: maintainerUp},
			ActionAny:    {Roles: writerUp},
		}},
		{Resource: ResourceProjectAccess, Actions: anyAction(maintainerUp)},
		{Resource: ResourceProjectDocument, Actions: map[string]generator.ActionRule{
			ActionRead:   {Roles: allRoles},
			ActionEdit:   {Roles: writerUp},
			ActionDelete: {Roles: maintainerUp},
		}},
		{Resource: ResourceTrigger, Actions: map[string]generator.ActionRule{
			ActionRead:   {Roles: allRoles},
			ActionCreate: {Roles: maintainerUp},
			ActionEdit:   {Roles: maintainerUp},
			ActionDelete: {Roles: maintainerUp},
		}},
		{Resource: ResourceUser, Actions: map[string]generator.ActionRule{
			ActionRead: {Roles: []string{roleSelf, roleMaintainer}},
			ActionEdit: {Roles: []string{roleSelf, roleMaintainer}},
		}},
		{Resource: ResourceWorkspace, Actions: map[string]generator.ActionRule{
			ActionList:          {Roles: []string{roleSelf, roleMaintainer}},
			ActionCreate:        {Roles: []string{roleSelf, roleMaintainer}},
			ActionEdit:          {Roles: []string{roleOwner}},
			ActionDelete:        {Roles: []string{roleOwner}},
			ActionManageMembers: {Roles: maintainerUp},
		}},
	}
}
