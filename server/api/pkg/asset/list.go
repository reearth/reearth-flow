package asset

import "github.com/reearth/reearth-flow/api/pkg/id"

type List []*Asset

func (l List) IDs() []id.AssetID {
	ids := make([]id.AssetID, 0, len(l))
	for _, a := range l {
		if a != nil {
			ids = append(ids, a.ID())
		}
	}
	return ids
}
