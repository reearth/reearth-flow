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
	roleWriter     = "writer"
	roleMaintainer = "maintainer"
	roleOwner      = "owner"
)

// DefineResources returns the flow RBAC policy as declarative resource rules.
// reearthx's generator.GeneratePolicies builds and sorts these into Cerbos
// policies (action ordering is deterministic via the generator), so only the
// resource→action→roles mapping matters here.
//
// Role vocabulary matches the accounts system and the UI: owner, maintainer,
// writer, reader, self. The backend previously used "editor", which is not a
// real workspace role (the accounts roles are owner/maintainer/writer/reader),
// so writers were effectively ungranted.
func DefineResources() []generator.ResourceRule {
	// Project content, deploy/run/debug and execution-data actions are granted to
	// writers, maintainers and owners. Per the Flow user-stories RBAC spec,
	// deploying a project and listing runs are Writer(Editor) + Maintainer + Owner,
	// and owner ⊇ maintainer is a universal invariant. These were previously
	// maintainer-only, which denied workspace owners and writers.
	writerMaintainerOwner := map[string]generator.ActionRule{
		ActionAny: {Roles: []string{roleWriter, roleMaintainer, roleOwner}},
	}
	// Privileged project-level actions (e.g. public sharing) are limited to
	// maintainers and owners (no writer).
	maintainerOwner := map[string]generator.ActionRule{
		ActionAny: {Roles: []string{roleMaintainer, roleOwner}},
	}
	return []generator.ResourceRule{
		{Resource: ResourceAsset, Actions: writerMaintainerOwner},
		{Resource: ResourceCMSAsset, Actions: writerMaintainerOwner},
		{Resource: ResourceCMSItem, Actions: writerMaintainerOwner},
		{Resource: ResourceCMSModel, Actions: writerMaintainerOwner},
		{Resource: ResourceCMSProject, Actions: writerMaintainerOwner},
		{Resource: ResourceDeployment, Actions: writerMaintainerOwner},
		{Resource: ResourceEdge, Actions: writerMaintainerOwner},
		{Resource: ResourceJob, Actions: writerMaintainerOwner},
		{Resource: ResourceLog, Actions: writerMaintainerOwner},
		{Resource: ResourceNode, Actions: writerMaintainerOwner},
		{Resource: ResourceParameter, Actions: writerMaintainerOwner},
		{Resource: ResourceProject, Actions: map[string]generator.ActionRule{
			ActionList:   {Roles: []string{roleSelf, roleMaintainer}},
			ActionCreate: {Roles: []string{roleMaintainer, roleOwner}},
			ActionEdit:   {Roles: []string{roleMaintainer, roleOwner}},
			ActionDelete: {Roles: []string{roleMaintainer, roleOwner}},
			ActionAny:    {Roles: []string{roleReader, roleWriter, roleOwner, roleMaintainer}},
		}},
		{Resource: ResourceProjectAccess, Actions: maintainerOwner},
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
