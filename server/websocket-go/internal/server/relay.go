package server

import (
	"context"
	"errors"

	"github.com/reearth/ygo/cluster"
)

// AttachRedisRelay binds a cluster.Relay to the underlying ygo provider so local
// doc/awareness changes mirror to sibling nodes and remote changes inject into
// local rooms. The provider drives the relay's lifecycle and Publish/Inject hooks.
func (s *Server) AttachRedisRelay(_ context.Context, r cluster.Relay) error {
	if r == nil {
		return errors.New("server: nil redis relay")
	}
	return s.ws.AttachRelay(r)
}
