package interactor

import (
	"encoding/json"
	"fmt"
	"maps"

	"github.com/reearth/reearth-flow/api/pkg/parameter"
	"github.com/reearth/reearthx/log"
)

type VariablesMode int

const (
	ModeExecuteDeployment VariablesMode = iota
	ModeAPIDriven
	ModeTimeDriven
)

func resolveVariables(
	mode VariablesMode,
	projectParams map[string]string,
	deploymentVars map[string]string,
	triggerVars map[string]string,
	requestVars map[string]string,
) map[string]string {
	out := map[string]string{}

	switch mode {
	case ModeExecuteDeployment:
		// ExecuteDeployment: request.variables ← deployment.variables ← project.parameters
		if projectParams != nil {
			maps.Copy(out, projectParams)
		}
		if deploymentVars != nil {
			maps.Copy(out, deploymentVars)
		}
		if requestVars != nil {
			maps.Copy(out, requestVars)
		}

	case ModeAPIDriven:
		// REST /run: request.with ← trigger.variables ← deployment.variables ← project.parameters
		if projectParams != nil {
			maps.Copy(out, projectParams)
		}
		if deploymentVars != nil {
			maps.Copy(out, deploymentVars)
		}
		if triggerVars != nil {
			maps.Copy(out, triggerVars)
		}
		if requestVars != nil {
			maps.Copy(out, requestVars)
		}

	case ModeTimeDriven:
		// REST /execute-scheduled: trigger.variables ← deployment.variables ← project.parameters
		if projectParams != nil {
			maps.Copy(out, projectParams)
		}
		if deploymentVars != nil {
			maps.Copy(out, deploymentVars)
		}
		if triggerVars != nil {
			maps.Copy(out, triggerVars)
		}
	}

	return out
}

func projectParametersToMap(pl *parameter.ParameterList) map[string]string {
	if pl == nil || len(*pl) == 0 {
		return nil
	}
	out := make(map[string]string, len(*pl))
	for _, p := range *pl {
		value := p.DefaultValue()

		if value == nil {
			continue
		}

		var s string

		switch p.Type() {
		case parameter.TypeGeometry, parameter.TypeArray, parameter.TypeChoice:
			b, err := json.Marshal(value)
			if err != nil {
				log.Debugf("failed to marshal parameter value: %v", err)
				continue
			}
			s = string(b)
		default:
			s = fmt.Sprintf("%v", value)
		}

		if s != "" {
			out[p.Name()] = s
		}
	}
	return out
}
