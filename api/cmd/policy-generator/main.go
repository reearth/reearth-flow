// cmd/policy-generator/main.go
package main

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/reearth/reearth-flow/api/internal/rbac"
	"gopkg.in/yaml.v2"
)

type CerbosPolicy struct {
	APIVersion     string         `yaml:"apiVersion"`
	ResourcePolicy ResourcePolicy `yaml:"resourcePolicy"`
}

type ResourcePolicy struct {
	Version  string `yaml:"version"`
	Resource string `yaml:"resource"`
	Rules    []Rule `yaml:"rules"`
}

type Rule struct {
	Actions []string `yaml:"actions"`
	Effect  string   `yaml:"effect"`
	Roles   []string `yaml:"roles"`
}

func main() {
	resources := rbac.DefineResources()

	for _, resource := range resources {
		policy := CerbosPolicy{
			APIVersion: "api.cerbos.dev/v1",
			ResourcePolicy: ResourcePolicy{
				Version:  "default",
				Resource: resource.Resource,
				Rules:    make([]Rule, 0, len(resource.Actions)),
			},
		}

		for _, action := range resource.Actions {
			roles := make([]string, 0, len(action.Roles))
			for _, role := range action.Roles {
				roles = append(roles, string(role))
			}

			rule := Rule{
				Actions: []string{string(action.Action)},
				Effect:  "EFFECT_ALLOW",
				Roles:   roles,
			}
			policy.ResourcePolicy.Rules = append(policy.ResourcePolicy.Rules, rule)
		}

		// ポリシーファイルの出力
		filename := strings.ReplaceAll(resource.Resource, ":", "_")
		outputPath := filepath.Join("policies", fmt.Sprintf("%s.yaml", filename))
		data, err := yaml.Marshal(policy)
		if err != nil {
			fmt.Printf("Error marshaling policy: %v\n", err)
			os.Exit(1)
		}

		if err := os.MkdirAll("policies", 0755); err != nil {
			fmt.Printf("Error creating directory: %v\n", err)
			os.Exit(1)
		}

		if err := os.WriteFile(outputPath, data, 0644); err != nil {
			fmt.Printf("Error writing file: %v\n", err)
			os.Exit(1)
		}
	}
}
