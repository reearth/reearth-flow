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
	"github.com/stretchr/testify/require"
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
	}
}

func TestListActionsWithSearch(t *testing.T) {
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
		assert.Contains(t, strings.ToLower(action.Name+" "+action.Description), "file")
	}
}

func TestGetSegregatedActions(t *testing.T) {
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
	assert.NotEmpty(t, response.ByCategory, "Should have at least one category")
	assert.NotEmpty(t, response.ByType, "Should have at least one type")

	uniqueActionsByCategory := make(map[string]bool)
	for _, actions := range response.ByCategory {
		for _, action := range actions {
			uniqueActionsByCategory[action.Name] = true
		}
	}

	totalByType := 0
	for _, actions := range response.ByType {
		totalByType += len(actions)
	}

	assert.Equal(t, len(uniqueActionsByCategory), totalByType,
		"Total unique actions (including Uncategorized) should match total actions in ByType")

	for typeName, typeActions := range response.ByType {
		for _, action := range typeActions {
			found := false
			for _, categoryActions := range response.ByCategory {
				for _, catAction := range categoryActions {
					if catAction.Name == action.Name {
						found = true
						break
					}
				}
				if found {
					break
				}
			}
			assert.True(t, found, "Action %s of type %s should be present in at least one category (including Uncategorized)", action.Name, typeName)
		}
	}

	uncategorizedActions, exists := response.ByCategory["Uncategorized"]
	assert.True(t, exists, "Uncategorized category should exist")
	if exists {
		routerFound := false
		for _, action := range uncategorizedActions {
			if action.Name == "Router" {
				routerFound = true
				break
			}
		}
		assert.True(t, routerFound, "Router action should be in the Uncategorized category")
	}
}

func TestGetActionDetails(t *testing.T) {
	e := echo.New()
	listReq := httptest.NewRequest(http.MethodGet, "/actions", nil)
	listRec := httptest.NewRecorder()
	listC := e.NewContext(listReq, listRec)

	err := listActions(listC)
	require.NoError(t, err)
	require.Equal(t, http.StatusOK, listRec.Code)

	var actionList []ActionSummary
	err = json.Unmarshal(listRec.Body.Bytes(), &actionList)
	require.NoError(t, err)
	require.NotEmpty(t, actionList, "No actions found in the list")

	firstAction := actionList[0]

	req := httptest.NewRequest(http.MethodGet, "/", nil)
	rec := httptest.NewRecorder()
	c := e.NewContext(req, rec)
	c.SetPath("/actions/:id")
	c.SetParamNames("id")
	c.SetParamValues(firstAction.Name)

	err = getActionDetails(c)
	assert.NoError(t, err)
	assert.Equal(t, http.StatusOK, rec.Code)

	var response Action
	err = json.Unmarshal(rec.Body.Bytes(), &response)
	assert.NoError(t, err)
	assert.Equal(t, firstAction.Name, response.Name)
	assert.Equal(t, firstAction.Description, response.Description)
	assert.Equal(t, firstAction.Type, string(response.Type))
	assert.Equal(t, firstAction.Categories, response.Categories)
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
