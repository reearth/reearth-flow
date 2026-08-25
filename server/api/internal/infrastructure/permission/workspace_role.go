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

// roleAllows reports whether a caller holding workspaceRole in the target
// workspace satisfies the policy rule for resource/action.
//
// It reads the same rbac.DefineResources() the Cerbos policies are generated
// from, and evaluates the principal the accounts service will send once the
// stale global roles are stripped: the workspace role plus self, with no
// global roles unioned in.
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

// workspaceRoleAllows re-evaluates resource/action against the caller's role in
// wsID alone. It can only deny: callers reach it having already been allowed by
// Cerbos, and it exists because the accounts service still unions each user's
// stale global roles into every check, so a workspace reader can arrive at
// Cerbos carrying maintainer or owner.
//
// TODO: remove once reearth-accounts#266 strips the stale global roles, after
// which Cerbos alone returns this same answer.
func (c *checker) workspaceRoleAllows(ctx context.Context, resource, action string, wsID accountsid.WorkspaceID) bool {
	u := adapter.User(ctx)
	if u == nil {
		return true // no user principal (integration, API trigger): not a workspace member, leave to Cerbos
	}

	// Cannot determine the role: keep the Cerbos verdict rather than deny a real
	// user. Logged because this is the guard's blind spot — a spike here means
	// it is silently not running, so it needs to be visible in production.
	ws, err := c.workspaceRepo.FindByID(ctx, wsID.String())
	if err != nil {
		log.Warnfc(ctx, "permission: workspace %s lookup failed, skipping the workspace-role guard for %s/%s: %v", wsID, resource, action, err)
		return true
	}
	// No membership at all is a data problem, not an answer: every caller would
	// be denied. Distinguish it from a populated list the caller is absent
	// from, which is a real non-member and a real deny.
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
