package gql

import (
	"fmt"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
)

func toStringMap(j gqlmodel.JSON) (map[string]string, error) {
	if j == nil {
		return nil, nil
	}

	raw := j
	out := make(map[string]string, len(raw))
	for k, v := range raw {
		s, ok := v.(string)
		if !ok {
			return nil, fmt.Errorf("variable %q must be string", k)
		}
		out[k] = s
	}
	return out, nil
}
