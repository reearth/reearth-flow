package mongodoc

import (
	"reflect"
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/parameter"
)

func TestNewParameter(t *testing.T) {
	pid := id.NewParameterID()
	projID := id.NewProjectID()
	now := time.Now().UTC()

	param, err := parameter.New().
		ID(pid).
		ProjectID(projID).
		Name("test-param").
		Type(parameter.TypeText).
		Required(true).
		Public(true).
		DefaultValue("some value").
		Index(5).
		CreatedAt(now).
		UpdatedAt(now).
		Build()
	if err != nil {
		t.Fatalf("unexpected error building parameter: %v", err)
	}

	doc, docID := NewParameter(param)
	if docID != pid.String() {
		t.Errorf("expected docID %s, got %s", pid.String(), docID)
	}

	if doc.ID != pid.String() {
		t.Errorf("expected doc ID %s, got %s", pid.String(), doc.ID)
	}
	if doc.Project != projID.String() {
		t.Errorf("expected project ID %s, got %s", projID.String(), doc.Project)
	}
	if doc.Name != "test-param" {
		t.Errorf("expected name 'test-param', got '%s'", doc.Name)
	}
	if doc.Type != "TEXT" {
		t.Errorf("expected type 'TEXT', got '%s'", doc.Type)
	}
	if doc.Required != true {
		t.Errorf("expected required to be true, got %v", doc.Required)
	}
	if doc.Public != true {
		t.Errorf("expected public to be true, got %v", doc.Public)
	}
	if doc.DefaultValue != "some value" {
		t.Errorf("expected value 'some value', got '%v'", doc.DefaultValue)
	}
	if doc.Index != 5 {
		t.Errorf("expected index 5, got %d", doc.Index)
	}
	if !doc.CreatedAt.Equal(now) {
		t.Errorf("expected createdAt %v, got %v", now, doc.CreatedAt)
	}
	if !doc.UpdatedAt.Equal(now) {
		t.Errorf("expected updatedAt %v, got %v", now, doc.UpdatedAt)
	}
}

func TestParameterDocument_Model(t *testing.T) {
	pid := id.NewParameterID()
	projID := id.NewProjectID()
	now := time.Now().UTC()

	doc := &ParameterDocument{
		CreatedAt:    now,
		ID:           pid.String(),
		Index:        3,
		Name:         "my-param",
		Project:      projID.String(),
		Required:     false,
		Public:       true,
		Type:         "NUMBER",
		UpdatedAt:    now,
		DefaultValue: 123.45,
	}

	param, err := doc.Model()
	if err != nil {
		t.Fatalf("unexpected error from Model(): %v", err)
	}
	if param == nil {
		t.Fatal("expected parameter, got nil")
	}

	if param.ID() != pid {
		t.Errorf("expected ID %s, got %s", pid.String(), param.ID().String())
	}
	if param.ProjectID() != projID {
		t.Errorf("expected projectID %s, got %s", projID.String(), param.ProjectID().String())
	}
	if param.Name() != "my-param" {
		t.Errorf("expected name 'my-param', got '%s'", param.Name())
	}
	if param.Type() != parameter.TypeNumber {
		t.Errorf("expected type NUMBER, got %s", param.Type())
	}
	if param.Required() != false {
		t.Errorf("expected required false, got %v", param.Required())
	}
	if param.Public() != true {
		t.Errorf("expected public true, got %v", param.Public())
	}
	if param.DefaultValue() != 123.45 {
		t.Errorf("expected value 123.45, got %v", param.DefaultValue())
	}
	if param.Index() != 3 {
		t.Errorf("expected index 3, got %d", param.Index())
	}
	if !param.CreatedAt().Equal(now) {
		t.Errorf("expected createdAt %v, got %v", now, param.CreatedAt())
	}
	if !param.UpdatedAt().Equal(now) {
		t.Errorf("expected updatedAt %v, got %v", now, param.UpdatedAt())
	}
}

func TestParameterDocument_Config(t *testing.T) {
	pid := id.NewParameterID()
	projID := id.NewProjectID()
	now := time.Now().UTC()

	// Test with config data
	configData := map[string]interface{}{
		"choices": []string{"option1", "option2", "option3"},
		"multiSelect": true,
		"placeholder": "Select an option",
	}

	param, err := parameter.New().
		ID(pid).
		ProjectID(projID).
		Name("choice-param").
		Type(parameter.TypeChoice).
		Required(false).
		Public(true).
		DefaultValue("option1").
		Config(configData).
		Index(0).
		CreatedAt(now).
		UpdatedAt(now).
		Build()
	if err != nil {
		t.Fatalf("unexpected error building parameter: %v", err)
	}

	// Test NewParameter conversion
	doc, docID := NewParameter(param)
	if docID != pid.String() {
		t.Errorf("expected docID %s, got %s", pid.String(), docID)
	}

	// Verify config is stored properly
	if doc.Config == nil {
		t.Fatal("expected config to be stored, got nil")
	}

	storedConfig, ok := doc.Config.(map[string]interface{})
	if !ok {
		t.Fatalf("expected config to be map[string]interface{}, got %T", doc.Config)
	}

	if !reflect.DeepEqual(storedConfig, configData) {
		t.Errorf("config data mismatch.\nExpected: %+v\nGot: %+v", configData, storedConfig)
	}

	// Test Model() conversion back to domain
	reconstructedParam, err := doc.Model()
	if err != nil {
		t.Fatalf("unexpected error from Model(): %v", err)
	}

	if reconstructedParam.Config() == nil {
		t.Fatal("expected reconstructed config to not be nil")
	}

	reconstructedConfig, ok := reconstructedParam.Config().(map[string]interface{})
	if !ok {
		t.Fatalf("expected reconstructed config to be map[string]interface{}, got %T", reconstructedParam.Config())
	}

	if !reflect.DeepEqual(reconstructedConfig, configData) {
		t.Errorf("reconstructed config data mismatch.\nExpected: %+v\nGot: %+v", configData, reconstructedConfig)
	}
}

func TestNewParameters(t *testing.T) {
	pid1 := id.NewParameterID()
	pid2 := id.NewParameterID()
	projID := id.NewProjectID()
	now := time.Now().UTC()

	param1, err := parameter.New().
		ID(pid1).
		ProjectID(projID).
		Name("param1").
		Type(parameter.TypeText).
		DefaultValue("v1").
		Index(0).
		CreatedAt(now).
		UpdatedAt(now).
		Build()
	if err != nil {
		t.Fatalf("unexpected error building param1: %v", err)
	}

	param2, err := parameter.New().
		ID(pid2).
		ProjectID(projID).
		Name("param2").
		Type(parameter.TypeNumber).
		DefaultValue(42).
		Index(1).
		CreatedAt(now).
		UpdatedAt(now).
		Build()
	if err != nil {
		t.Fatalf("unexpected error building param2: %v", err)
	}

	params := parameter.ParameterList{param1, param2}
	docs, ids := NewParameters(params)

	if len(docs) != 2 {
		t.Fatalf("expected 2 docs, got %d", len(docs))
	}
	if len(ids) != 2 {
		t.Fatalf("expected 2 ids, got %d", len(ids))
	}

	if ids[0] != pid1.String() || ids[1] != pid2.String() {
		t.Errorf("expected IDs [%s, %s], got [%s, %s]", pid1.String(), pid2.String(), ids[0], ids[1])
	}

	doc1, ok := docs[0].(*ParameterDocument)
	if !ok {
		t.Fatalf("expected doc1 to be *ParameterDocument, got %T", docs[0])
	}
	doc2, ok := docs[1].(*ParameterDocument)
	if !ok {
		t.Fatalf("expected doc2 to be *ParameterDocument, got %T", docs[1])
	}

	if doc1.Name != "param1" || doc1.DefaultValue != "v1" || doc1.Type != "TEXT" {
		t.Errorf("doc1 did not match expected values")
	}
	if doc2.Name != "param2" || !reflect.DeepEqual(doc2.DefaultValue, 42) || doc2.Type != "NUMBER" {
		t.Errorf("doc2 did not match expected values")
	}
}
