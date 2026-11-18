package interactor

import (
	"context"
	"errors"
	"fmt"
	"io"
	"net/url"
	"path"

	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/pkg/parameter"
	"github.com/reearth/reearth-flow/api/pkg/variable"
	"gopkg.in/yaml.v3"
)

// readWorkflowWith reads the `with` block of a deployment's workflow file, which
// the worker uses for a run that sets no values of its own. The file may be JSON
// or YAML, and the YAML parser reads both.
func readWorkflowWith(ctx context.Context, files gateway.File, workflowURL string) (map[string]any, error) {
	u, err := url.Parse(workflowURL)
	if err != nil {
		return nil, fmt.Errorf("parse workflow url: %w", err)
	}
	name := path.Base(u.Path)
	if name == "." || name == "/" {
		return nil, gateway.ErrInvalidFile
	}

	r, err := files.ReadWorkflow(ctx, name)
	if err != nil {
		return nil, fmt.Errorf("read workflow %s: %w", name, err)
	}
	defer func() { _ = r.Close() }()

	var wf struct {
		With map[string]any `yaml:"with"`
	}
	if err := yaml.NewDecoder(r).Decode(&wf); err != nil && !errors.Is(err, io.EOF) {
		return nil, fmt.Errorf("decode workflow %s: %w", name, err)
	}
	return wf.With, nil
}

// workflowWithVariables types a workflow file's `with` values by the
// deployment's variables, for the job's record. A value its type cannot read is
// kept as the file holds it, and an undefined key is recorded as text.
func workflowWithVariables(defs []*parameter.Parameter, with map[string]any) map[string]variable.Variable {
	if len(with) == 0 {
		return nil
	}
	types := make(map[string]parameter.Type, len(defs))
	for _, p := range defs {
		if p != nil {
			types[p.Name()] = p.Type()
		}
	}

	out := make(map[string]variable.Variable, len(with))
	for k, v := range with {
		t, defined := types[k]
		if !defined {
			t = parameter.TypeText
		}
		if v != nil {
			if c, ok := coerceValue(v, t); ok {
				v = c
			}
		}
		out[k] = variable.Variable{Key: k, Type: t, Value: v}
	}
	return out
}
