package gqlmodel

import (
	"github.com/reearth/reearth-flow/api/pkg/parameter"
	"go.mongodb.org/mongo-driver/bson/primitive"
)

func ToParameter(p *parameter.Parameter) *Parameter {
	if p == nil {
		return nil
	}

	return &Parameter{
		CreatedAt:    p.CreatedAt(),
		ID:           IDFrom(p.ID()),
		Index:        p.Index(),
		Name:         p.Name(),
		ProjectID:    IDFrom(p.ProjectID()),
		Required:     p.Required(),
		Public:       p.Public(),
		Type:         ToParameterType(p.Type()),
		UpdatedAt:    p.UpdatedAt(),
		DefaultValue: p.DefaultValue(),
		Config:       convertToJSON(p.Config()),
	}
}

func ToParameters(params *parameter.ParameterList) []*Parameter {
	if params == nil {
		return nil
	}

	res := make([]*Parameter, 0, len(*params))
	for _, p := range *params {
		if p != nil {
			res = append(res, ToParameter(p))
		}
	}
	return res
}

func ToParameterType(t parameter.Type) ParameterType {
	switch t {
	case parameter.TypeChoice:
		return ParameterTypeChoice
	case parameter.TypeColor:
		return ParameterTypeColor
	case parameter.TypeDatetime:
		return ParameterTypeDatetime
	case parameter.TypeFileFolder:
		return ParameterTypeFileFolder
	case parameter.TypeText:
		return ParameterTypeText
	case parameter.TypeYesNo:
		return ParameterTypeYesNo
	case parameter.TypeNumber:
		return ParameterTypeNumber
	// case parameter.TypeMessage:
	// 	return ParameterTypeMessage
	// case parameter.TypePassword:
	// 	return ParameterTypePassword
	// case parameter.TypeAttributeName:
	// 	return ParameterTypeAttributeName
	// case parameter.TypeCoordinateSystem:
	// 	return ParameterTypeCoordinateSystem
	// case parameter.TypeDatabaseConnection:
	// 	return ParameterTypeDatabaseConnection
	// case parameter.TypeGeometry:
	// 	return ParameterTypeGeometry
	// case parameter.TypeReprojectionFile:
	// 	return ParameterTypeReprojectionFile
	// case parameter.TypeWebConnection:
	// 	return ParameterTypeWebConnection
	default:
		return ParameterTypeText
	}
}

func FromParameterType(t ParameterType) parameter.Type {
	switch t {
	case ParameterTypeChoice:
		return parameter.TypeChoice
	case ParameterTypeColor:
		return parameter.TypeColor
	case ParameterTypeDatetime:
		return parameter.TypeDatetime
	case ParameterTypeFileFolder:
		return parameter.TypeFileFolder
	case ParameterTypeNumber:
		return parameter.TypeNumber
	case ParameterTypeText:
		return parameter.TypeText
	case ParameterTypeYesNo:
		return parameter.TypeYesNo
	// case ParameterTypeMessage:
	// 	return parameter.TypeMessage
	// case ParameterTypePassword:
	// 	return parameter.TypePassword
	// case ParameterTypeAttributeName:
	// 	return parameter.TypeAttributeName
	// case ParameterTypeCoordinateSystem:
	// 	return parameter.TypeCoordinateSystem
	// case ParameterTypeDatabaseConnection:
	// 	return parameter.TypeDatabaseConnection
	// case ParameterTypeGeometry:
	// 	return parameter.TypeGeometry
	// case ParameterTypeReprojectionFile:
	// 	return parameter.TypeReprojectionFile
	// case ParameterTypeWebConnection:
	// 	return parameter.TypeWebConnection
	default:
		return parameter.TypeText
	}
}

func convertToJSON(val interface{}) JSON {
	if val == nil {
		return nil
	}

	// Handle MongoDB's primitive.D type (ordered document)
	if d, ok := val.(primitive.D); ok {
		result := make(map[string]interface{})
		for _, elem := range d {
			result[elem.Key] = elem.Value
		}
		return JSON(result)
	}

	// If it's already a map[string]any, use it directly
	if m, ok := val.(map[string]any); ok {
		return JSON(m)
	}

	// Check for map[string]interface{} (common MongoDB type)
	if m, ok := val.(map[string]interface{}); ok {
		return JSON(m)
	}

	// For other types, use UnmarshalJSON to convert consistently
	json, err := UnmarshalJSON(val)
	if err != nil {
		return nil
	}

	return json
}
