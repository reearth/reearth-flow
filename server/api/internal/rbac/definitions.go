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
	ResourceCMSAsset      = "cms_asset"
	ResourceCMSItem       = "cms_item"
	ResourceCMSModel      = "cms_model"
	ResourceCMSProject    = "cms_project"
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

// DefineResources returns the flow RBAC policy as declarative resource rules.
// reearthx's generator.GeneratePolicies builds and sorts these into Cerbos
// policies (action ordering is deterministic via the generator), so only the
// resource→action→roles mapping matters here.
func DefineResources() []generator.ResourceRule {
	maintainerOnly := map[string]generator.ActionRule{
		ActionAny: {Roles: []string{roleMaintainer}},
	}
	return []generator.ResourceRule{
		{Resource: ResourceAsset, Actions: maintainerOnly},
		{Resource: ResourceCMSAsset, Actions: maintainerOnly},
		{Resource: ResourceCMSItem, Actions: maintainerOnly},
		{Resource: ResourceCMSModel, Actions: maintainerOnly},
		{Resource: ResourceCMSProject, Actions: maintainerOnly},
		{Resource: ResourceDeployment, Actions: maintainerOnly},
		{Resource: ResourceEdge, Actions: maintainerOnly},
		{Resource: ResourceJob, Actions: maintainerOnly},
		{Resource: ResourceLog, Actions: maintainerOnly},
		{Resource: ResourceNode, Actions: maintainerOnly},
		{Resource: ResourceParameter, Actions: maintainerOnly},
		{Resource: ResourceProject, Actions: map[string]generator.ActionRule{
			ActionList:   {Roles: []string{roleSelf, roleMaintainer}},
			ActionCreate: {Roles: []string{roleMaintainer, roleOwner}},
			ActionEdit:   {Roles: []string{roleMaintainer, roleOwner}},
			ActionDelete: {Roles: []string{roleMaintainer, roleOwner}},
			ActionAny:    {Roles: []string{roleReader, roleEditor, roleOwner, roleMaintainer}},
		}},
		{Resource: ResourceProjectAccess, Actions: maintainerOnly},
		{Resource: ResourceTrigger, Actions: map[string]generator.ActionRule{
			ActionCreate: {Roles: []string{roleMaintainer, roleOwner}},
			ActionEdit:   {Roles: []string{roleMaintainer, roleOwner}},
			ActionDelete: {Roles: []string{roleMaintainer, roleOwner}},
			ActionAny:    {Roles: []string{roleSelf, roleMaintainer}},
		}},
		{Resource: ResourceUser, Actions: map[string]generator.ActionRule{
			ActionRead: {Roles: []string{roleSelf, roleMaintainer}},
			ActionEdit: {Roles: []string{roleSelf, roleMaintainer}},
		}},
		{Resource: ResourceWorkspace, Actions: map[string]generator.ActionRule{
			ActionList:   {Roles: []string{roleSelf, roleMaintainer}},
			ActionCreate: {Roles: []string{roleSelf, roleMaintainer}},
			ActionEdit:   {Roles: []string{roleOwner, roleMaintainer}},
			ActionDelete: {Roles: []string{roleOwner, roleMaintainer}},
		}},
	}
}
