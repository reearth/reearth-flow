package app

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/labstack/echo/v4"
	"github.com/stretchr/testify/assert"
)

func resetTestData() {
	actionsData = ActionsData{}
	actionsDataMap = make(map[string]ActionsData)
}

func TestLoadActionsData(t *testing.T) {
	tests := []struct {
		name    string
		lang    string
		wantErr bool
	}{
		{"Default language", "", false},
		{"En1ish", "en", false},
		{"Japanese", "ja", false},
		{"Invalid language", "invalid", true},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			resetTestData()
			err := loadActionsData(tt.lang)
			if tt.wantErr {
				assert.Error(t, err)
			} else {
				assert.NoError(t, err)
				assert.NotEmpty(t, actionsData.Actions)

				// Verify cache
				assert.NotNil(t, actionsDataMap[tt.lang])
				assert.Equal(t, actionsData, actionsDataMap[tt.lang])
			}
		})
	}
}

func TestListActions(t *testing.T) {
	e := echo.New()
	tests := []struct {
		name     string
		query    string
		lang     string
		wantCode int
	}{
		{"Default language", "", "", http.StatusOK},
		{"With language", "", "en", http.StatusOK},
		{"Japanese language", "", "ja", http.StatusOK},
		{"Invalid language", "", "invalid", http.StatusBadRequest},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			resetTestData()
			req := httptest.NewRequest(http.MethodGet, "/actions?lang="+tt.lang+tt.query, nil)
			rec := httptest.NewRecorder()
			c := e.NewContext(req, rec)

			err := listActions(c)
			assert.NoError(t, err)
			assert.Equal(t, tt.wantCode, rec.Code)

			if tt.wantCode == http.StatusOK {
				var response []ActionSummary
				err := json.Unmarshal(rec.Body.Bytes(), &response)
				assert.NoError(t, err)
				assert.NotEmpty(t, response)
			}
		})
	}
}

func TestGetSegregatedActions(t *testing.T) {
	testActions := []Action{
		{
			Name:        "CsvWriter",
			Type:        ActionTypeSink,
			Description: "Writes features to a CSV file",
			Categories:  []string{"File"},
		},
		{
			Name:        "Router",
			Type:        ActionTypeProcessor,
			Description: "Action for port forwarding",
			Categories:  []string{},
		},
	}

	tests := []struct {
		name     string
		lang     string
		wantCode int
	}{
		{"Default language", "", http.StatusOK},
		{"English", "en", http.StatusOK},
		{"Japanese", "ja", http.StatusOK},
		{"Invalid language", "invalid", http.StatusBadRequest},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			resetTestData()
			actionsData = ActionsData{Actions: testActions}
			actionsDataMap[tt.lang] = actionsData

			e := echo.New()
			req := httptest.NewRequest(http.MethodGet, "/actions/segregated?lang="+tt.lang, nil)
			rec := httptest.NewRecorder()
			c := e.NewContext(req, rec)

			err := getSegregatedActions(c)
			assert.NoError(t, err)
			assert.Equal(t, tt.wantCode, rec.Code)

			if tt.wantCode == http.StatusOK {
				var response SegregatedActions
				err = json.Unmarshal(rec.Body.Bytes(), &response)
				assert.NoError(t, err)
				assert.NotEmpty(t, response.ByCategory)
				assert.NotEmpty(t, response.ByType)
			}
		})
	}
}

func TestGetActionDetails(t *testing.T) {
	testAction := Action{
		Name:        "TestAction",
		Type:        ActionTypeProcessor,
		Description: "Test action description",
		Categories:  []string{"TestCategory"},
	}

	tests := []struct {
		name     string
		lang     string
		id       string
		wantCode int
	}{
		{"Default language", "", testAction.Name, http.StatusOK},
		{"English", "en", testAction.Name, http.StatusOK},
		{"Japanese", "ja", testAction.Name, http.StatusOK},
		{"Invalid language", "invalid", testAction.Name, http.StatusBadRequest},
		{"Not found", "en", "NonExistent", http.StatusNotFound},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			resetTestData()
			actionsData = ActionsData{Actions: []Action{testAction}}
			actionsDataMap[tt.lang] = actionsData

			e := echo.New()
			req := httptest.NewRequest(http.MethodGet, "/?lang="+tt.lang, nil)
			rec := httptest.NewRecorder()
			c := e.NewContext(req, rec)
			c.SetPath("/actions/:id")
			c.SetParamNames("id")
			c.SetParamValues(tt.id)

			err := getActionDetails(c)
			assert.NoError(t, err)
			assert.Equal(t, tt.wantCode, rec.Code)

			if tt.wantCode == http.StatusOK {
				var response Action
				err = json.Unmarshal(rec.Body.Bytes(), &response)
				assert.NoError(t, err)
				assert.Equal(t, testAction.Name, response.Name)
			}
		})
	}
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
