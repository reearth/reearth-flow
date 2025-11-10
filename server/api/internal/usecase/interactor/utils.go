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
	vals := map[string]string{}

	switch mode {
	case ModeExecuteDeployment:
		// ExecuteDeployment: request.variables ← deployment.variables ← project.parameters
		if projectParams != nil {
			maps.Copy(vals, projectParams)
		}
		if deploymentVars != nil {
			maps.Copy(vals, deploymentVars)
		}
		if requestVars != nil {
			maps.Copy(vals, requestVars)
		}

	case ModeAPIDriven:
		// REST /run: request.with ← trigger.variables ← deployment.variables ← project.parameters
		if projectParams != nil {
			maps.Copy(vals, projectParams)
		}
		if deploymentVars != nil {
			maps.Copy(vals, deploymentVars)
		}
		if triggerVars != nil {
			maps.Copy(vals, triggerVars)
		}
		if requestVars != nil {
			maps.Copy(vals, requestVars)
		}

	case ModeTimeDriven:
		// REST /execute-scheduled: trigger.variables ← deployment.variables ← project.parameters
		if projectParams != nil {
			maps.Copy(vals, projectParams)
		}
		if deploymentVars != nil {
			maps.Copy(vals, deploymentVars)
		}
		if triggerVars != nil {
			maps.Copy(vals, triggerVars)
		}
	}

	return vals
}

func projectParametersToMap(pl *parameter.ParameterList) map[string]string {
	if pl == nil || len(*pl) == 0 {
		return nil
	}
	vals := make(map[string]string, len(*pl))
	for _, p := range *pl {
		value := p.DefaultValue()

		if value == nil {
			continue
		}

		var s string

		switch p.Type() {
		case parameter.TypeArray, parameter.TypeChoice, parameter.TypeDatetime:
			b, err := json.Marshal(value)
			if err != nil {
				log.Debugf("failed to marshal parameter value: %v", err)
				continue
			}
			s = string(b)
		case parameter.TypeText, parameter.TypeNumber, parameter.TypeYesNo, parameter.TypeColor:
			s = fmt.Sprintf("%v", value)
		default:
			log.Debugf("unsupported parameter type: %s (%v)", p.Name(), p.Type())
			continue
		}

		vals[p.Name()] = s
	}
	return vals
}
