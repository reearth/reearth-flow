package app

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strings"
	"sync"
	"testing"

	"github.com/labstack/echo/v4"
	"github.com/stretchr/testify/assert"
)

func TestLoadActionsData(t *testing.T) {
	actionsData = ActionsData{}
	once = sync.Once{}

	err := loadActionsData()
	assert.NoError(t, err)
	assert.NotEmpty(t, actionsData.Actions)
}

func TestListActions(t *testing.T) {
	e := echo.New()
	req := httptest.NewRequest(http.MethodGet, "/actions", nil)
	rec := httptest.NewRecorder()
	c := e.NewContext(req, rec)

	if assert.NoError(t, listActions(c)) {
		assert.Equal(t, http.StatusOK, rec.Code)

		var response []ActionSummary
		err := json.Unmarshal(rec.Body.Bytes(), &response)
		assert.NoError(t, err)
		assert.NotEmpty(t, response)

		firstAction := response[0]
		assert.NotEmpty(t, firstAction.Name)
		assert.NotEmpty(t, firstAction.Type)
		assert.NotEmpty(t, firstAction.Description)
	}
}

func TestListActionsWithSearch(t *testing.T) {
	originalData := actionsData
	defer func() { actionsData = originalData }()

	actionsData = ActionsData{
		Actions: []Action{
			{
				Name:        "FileWriter",
				Description: "Writes features to a file",
				Type:        ActionTypeSink,
				Categories:  []string{"File"},
			},
			{
				Name:        "OtherAction",
				Description: "Some other action",
				Type:        ActionTypeProcessor,
			},
		},
	}

	e := echo.New()
	req := httptest.NewRequest(http.MethodGet, "/actions?q=file", nil)
	rec := httptest.NewRecorder()
	c := e.NewContext(req, rec)

	err := listActions(c)
	assert.NoError(t, err)
	assert.Equal(t, http.StatusOK, rec.Code)

	var response []ActionSummary
	err = json.Unmarshal(rec.Body.Bytes(), &response)
	assert.NoError(t, err)
	assert.NotEmpty(t, response, "Search should return at least one result")

	for _, action := range response {
		lowercaseContent := strings.ToLower(action.Name + " " + action.Description)
		assert.Contains(t, lowercaseContent, "file",
			"Each result should contain 'file' in name or description")
	}
}

func TestGetSegregatedActions(t *testing.T) {
	originalData := actionsData
	defer func() { actionsData = originalData }()

	actionsData = ActionsData{
		Actions: []Action{
			{
				Name:        "FileWriter",
				Type:        ActionTypeSink,
				Description: "Writes features to a file",
				Categories:  []string{"File"},
			},
			{
				Name:        "Router",
				Type:        ActionTypeProcessor,
				Description: "Action for last port forwarding for sub-workflows.",
				Categories:  []string{},
			},
		},
	}

	e := echo.New()
	req := httptest.NewRequest(http.MethodGet, "/actions/segregated", nil)
	rec := httptest.NewRecorder()
	c := e.NewContext(req, rec)

	err := getSegregatedActions(c)
	assert.NoError(t, err)
	assert.Equal(t, http.StatusOK, rec.Code)

	var response SegregatedActions
	err = json.Unmarshal(rec.Body.Bytes(), &response)
	assert.NoError(t, err)

	assert.NotEmpty(t, response.ByCategory)
	assert.NotEmpty(t, response.ByType)

	assert.Contains(t, response.ByCategory, "File")
	assert.Contains(t, response.ByCategory, "Uncategorized")
	assert.Contains(t, response.ByType, string(ActionTypeSink))
	assert.Contains(t, response.ByType, string(ActionTypeProcessor))

	uncategorizedActions := response.ByCategory["Uncategorized"]
	routerFound := false
	for _, action := range uncategorizedActions {
		if action.Name == "Router" {
			routerFound = true
			break
		}
	}
	assert.True(t, routerFound, "Router should be in Uncategorized category")
}

func TestGetActionDetails(t *testing.T) {
	originalData := actionsData
	defer func() { actionsData = originalData }()

	testAction := Action{
		Name:        "TestAction",
		Type:        ActionTypeProcessor,
		Description: "Test action description",
		Categories:  []string{"TestCategory"},
	}

	actionsData = ActionsData{
		Actions: []Action{testAction},
	}

	e := echo.New()
	req := httptest.NewRequest(http.MethodGet, "/", nil)
	rec := httptest.NewRecorder()
	c := e.NewContext(req, rec)
	c.SetPath("/actions/:id")
	c.SetParamNames("id")
	c.SetParamValues(testAction.Name)

	err := getActionDetails(c)
	assert.NoError(t, err)
	assert.Equal(t, http.StatusOK, rec.Code)

	var response Action
	err = json.Unmarshal(rec.Body.Bytes(), &response)
	assert.NoError(t, err)
	assert.Equal(t, testAction.Name, response.Name)
	assert.Equal(t, testAction.Description, response.Description)
	assert.Equal(t, testAction.Type, response.Type)
	assert.Equal(t, testAction.Categories, response.Categories)
}

func TestGetActionDetailsNotFound(t *testing.T) {
	e := echo.New()
	req := httptest.NewRequest(http.MethodGet, "/", nil)
	rec := httptest.NewRecorder()
	c := e.NewContext(req, rec)
	c.SetPath("/actions/:id")
	c.SetParamNames("id")
	c.SetParamValues("NonExistentAction")

	err := getActionDetails(c)
	assert.NoError(t, err)
	assert.Equal(t, http.StatusNotFound, rec.Code)

	var response map[string]string
	err = json.Unmarshal(rec.Body.Bytes(), &response)
	assert.NoError(t, err)
	assert.Equal(t, "Action not found", response["error"])
}

func TestMatchesSearch(t *testing.T) {
	action := Action{
		Name:        "TestAction",
		Type:        ActionTypeProcessor,
		Description: "This is a test action",
		Categories:  []string{"TestCategory", "AnotherCategory"},
	}

	tests := []struct {
		name       string
		query      string
		category   string
		actionType string
		want       bool
	}{
		{"Full match", "TestAction", "", "", true},
		{"Partial match name", "Test", "", "", true},
		{"Partial match description", "test action", "", "", true},
		{"Match category", "", "TestCategory", "", true},
		{"Match type", "", "", "processor", true},
		{"No match", "NonExistent", "", "", false},
		{"Wrong category", "", "WrongCategory", "", false},
		{"Wrong type", "", "", "source", false},
		{"Partial match with type and category", "Test", "TestCategory", "processor", true},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := matchesSearch(action, tt.query, tt.category, tt.actionType)
			assert.Equal(t, tt.want, got)
		})
	}
}

func TestPartialMatch(t *testing.T) {
	tests := []struct {
		name  string
		s     string
		query string
		want  bool
	}{
		{"Full match", "Test String", "Test String", true},
		{"Partial match", "Test String", "Test", true},
		{"Multiple word match", "Test String Example", "Test Example", true},
		{"Case insensitive", "Test String", "test string", true},
		{"No match", "Test String", "Nonexistent", false},
		{"Partial word no match", "TestString", "String Test", false},
		{"Partial word match", "TestString", "Test", true},
		{"Multiple partial word match", "TestString ExampleWord", "Test Exam", true},
		{"Order independent", "First Second Third", "Third First", true},
		{"Substring in middle no match", "ThisIsALongString", "IsA", false},
		{"Multiple words in one no match", "ThisIsALongString", "This Long", false},
		{"Split word no match", "TestString", "Test String", false},
		{"Exact word start required", "TestString", "Str", false},
		{"Partial match at word start", "Test String", "Te Str", true},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := partialMatch(tt.s, tt.query)
			assert.Equal(t, tt.want, got, "partialMatch(%v, %v)", tt.s, tt.query)
		})
	}
}

func TestContainsCaseInsensitive(t *testing.T) {
	slice := []string{"Test", "Sample", "Example"}

	tests := []struct {
		name string
		s    string
		want bool
	}{
		{"Exact match", "Test", true},
		{"Case insensitive match", "test", true},
		{"No match", "Nonexistent", false},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := containsCaseInsensitive(slice, tt.s)
			assert.Equal(t, tt.want, got)
		})
	}
}
