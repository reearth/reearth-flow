package interactor

import (
	"context"
	"time"

	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearthx/log"
)

// actionVersionTimeout bounds how long a debug run or a deployment waits for its
// version. A save cut short loses nothing, since the websocket server's next save
// includes the changes.
const actionVersionTimeout = 20 * time.Second

// saveActionVersion saves the project's workflow as a version labelled label
// when an action such as a debug run or a deployment uses it. The version is a
// record of the action, not part of it, so a failure is logged and the action
// goes on.
func saveActionVersion(ctx context.Context, client interfaces.WebsocketClient, projectID, label string) {
	if client == nil {
		return
	}
	ctx, cancel := context.WithTimeout(ctx, actionVersionTimeout)
	defer cancel()
	if _, err := client.SaveVersion(ctx, projectID, label); err != nil && !interfaces.IsVersionSaveSkipped(err) {
		log.Warnfc(ctx, "project %s: saving the %q version failed; the next version will include its changes: %v", projectID, label, err)
	}
}
