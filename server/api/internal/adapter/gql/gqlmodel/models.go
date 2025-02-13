package gqlmodel

import (
	"encoding/json"
	"fmt"
	"io"

	"github.com/99designs/gqlgen/graphql"
	"github.com/reearth/reearthx/log"
)

type JSON map[string]any

func MarshalJSON(b JSON) graphql.Marshaler {
	return graphql.WriterFunc(func(w io.Writer) {
		byteData, err := json.Marshal(b)
		if err != nil {
			log.Fatalf("failed to marshal JSON %v\n", string(byteData))
		}
		_, err = w.Write(byteData)
		if err != nil {
			log.Fatalf("failed to write to io.Writer: %v\n", string(byteData))
		}
	})
}

func UnmarshalJSON(v interface{}) (JSON, error) {
	byteData, err := json.Marshal(v)
	if err != nil {
		return JSON{}, fmt.Errorf("failed while marshalling scheme")
	}
	tmp := make(map[string]interface{})
	err = json.Unmarshal(byteData, &tmp)
	if err != nil {
		return JSON{}, fmt.Errorf("failed while unmarshalling scheme")
	}
	return tmp, nil
}
