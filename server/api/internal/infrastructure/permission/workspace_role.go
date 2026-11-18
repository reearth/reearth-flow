package permission

import (
	"context"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/reearth/reearthx/log"
)

// selfRole is granted to any authenticated member acting on their own behalf,
// so a rule naming it is satisfied by any member regardless of workspace role.
const selfRole = "self"

// roleAllows reports whether workspaceRole satisfies the rule for
// resource/action in rbac.DefineResources(), which the Cerbos policies are
// generated from. It evaluates the workspace role plus self, without the stale
// global roles the accounts service still unions in.
func roleAllows(resource, action, workspaceRole string) bool {
	for _, r := range rbac.DefineResources() {
		if r.Resource != resource {
			continue
		}
		rule, ok := r.Actions[action]
		if !ok {
			return false // an action with no rule is denied for everyone
		}
		for _, granted := range rule.Roles {
			if granted == selfRole || granted == workspaceRole {
				return true
			}
		}
		return false
	}
	return false
}

// workspaceRoleAllows re-checks a Cerbos allow against the caller's role in wsID
// alone, because the accounts service unions stale global roles into every check.
// TODO: remove once reearth-accounts#266 strips the stale global roles.
func (c *checker) workspaceRoleAllows(ctx context.Context, resource, action string, wsID accountsid.WorkspaceID) bool {
	u := adapter.User(ctx)
	if u == nil {
		return true // no user principal (integration, API trigger): not a workspace member, leave to Cerbos
	}

	// When the role cannot be determined (failed lookup, no workspace, or no
	// membership data), the guard keeps the Cerbos allow and logs a warning.
	ws, err := c.workspaceRepo.FindByID(ctx, wsID.String())
	if err != nil {
		log.Warnfc(ctx, "permission: workspace %s lookup failed, skipping the workspace-role guard for %s/%s: %v", wsID, resource, action, err)
		return true
	}
	if ws == nil {
		log.Warnfc(ctx, "permission: workspace %s not found, skipping the workspace-role guard for %s/%s", wsID, resource, action)
		return true
	}
	members := ws.Members()
	if members == nil || members.Count() == 0 {
		log.Warnfc(ctx, "permission: workspace %s has no membership data, skipping the workspace-role guard for %s/%s", wsID, resource, action)
		return true
	}

	role := string(members.UserRole(accountsid.UserID(u.ID())))
	if role == "" {
		log.Warnfc(ctx, "permission: caller is not a member of workspace %s; denying %s/%s", wsID, resource, action)
		return false
	}

	if !roleAllows(resource, action, role) {
		log.Warnfc(ctx, "permission: workspace role %q does not allow %s/%s in workspace %s", role, resource, action, wsID)
		return false
	}
	return true
}
