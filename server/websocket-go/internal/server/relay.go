package server

import (
	"context"
	"errors"

	"github.com/reearth/ygo/cluster"
)

// AttachRedisRelay binds a cluster.Relay to the ygo provider, which drives its
// lifecycle and mirrors doc and awareness changes between this node and its
// siblings. A relay that reports lost updates has its rooms republished after one.
func (s *Server) AttachRedisRelay(_ context.Context, r cluster.Relay) error {
	if r == nil {
		return errors.New("server: nil redis relay")
	}
	if err := s.ws.AttachRelay(r); err != nil {
		return err
	}
	s.relay = r
	s.gaps, _ = r.(gapTaker)
	return nil
}
