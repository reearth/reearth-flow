package app

import (
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"strings"
	"sync"

	"github.com/labstack/echo/v4"
)

type ActionType string

const (
	ActionTypeProcessor ActionType = "processor"
	ActionTypeSource    ActionType = "source"
	ActionTypeSink      ActionType = "sink"
)

type Action struct {
	Parameter   map[string]interface{} `json:"parameter"`
	Name        string                 `json:"name"`
	Type        ActionType             `json:"type"`
	Description string                 `json:"description"`
	InputPorts  []string               `json:"inputPorts"`
	OutputPorts []string               `json:"outputPorts"`
	Categories  []string               `json:"categories"`
	Builtin     bool                   `json:"builtin"`
}

func (a *Action) Validate() error {
	if a.Name == "" {
		return errors.New("action name cannot be empty")
	}
	if a.Type == "" {
		return errors.New("action type cannot be empty")
	}
	if !isValidActionType(a.Type) {
		return errors.New("invalid action type")
	}
	return nil
}

func isValidActionType(t ActionType) bool {
	switch t {
	case ActionTypeProcessor, ActionTypeSource, ActionTypeSink:
		return true
	default:
		return false
	}
}

type ActionsData struct {
	Actions []Action `json:"actions"`
}

func (ad *ActionsData) Validate() error {
	for _, action := range ad.Actions {
		if err := action.Validate(); err != nil {
			return errors.New("invalid action: " + action.Name + ": " + err.Error())
		}
	}
	return nil
}

type ActionSummary struct {
	Name        string   `json:"name"`
	Description string   `json:"description"`
	Type        string   `json:"type"`
	Categories  []string `json:"categories"`
}

type SegregatedActions struct {
	ByCategory map[string][]ActionSummary `json:"byCategory"`
	ByType     map[string][]ActionSummary `json:"byType"`
}

var (
	actionsData    ActionsData
	actionsDataMap = make(map[string]ActionsData)
	mutex          sync.RWMutex
	supportedLangs = map[string]bool{
		"en": true,
		"es": true,
		"fr": true,
		"ja": true,
		"zh": true,
	}
)

func loadActionsData(lang string) error {
	if lang != "" && !supportedLangs[lang] {
		return fmt.Errorf("unsupported language: %s", lang)
	}

	cacheKey := lang

	// Try to get from cache first using read lock
	mutex.RLock()
	if data, exists := actionsDataMap[cacheKey]; exists {
		actionsData = data
		mutex.RUnlock()
		return nil
	}
	mutex.RUnlock()

	// If not in cache, acquire write lock
	mutex.Lock()
	defer mutex.Unlock()

	// Double-check after acquiring write lock
	if data, exists := actionsDataMap[cacheKey]; exists {
		actionsData = data
		return nil
	}

	baseURL := "https://raw.githubusercontent.com/reearth/reearth-flow/main/engine/schema/"
	filename := "actions.json"
	if lang != "" {
		filename = fmt.Sprintf("actions_%s.json", lang)
	}

	resp, err := http.Get(baseURL + filename)
	if err != nil {
		return err
	}
	defer func() {
		if err := resp.Body.Close(); err != nil {
			fmt.Println("Error closing response body:", err)
		}
	}()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return err
	}

	var newData ActionsData
	if err := json.Unmarshal(body, &newData); err != nil {
		return err
	}

	if err := newData.Validate(); err != nil {
		return err
	}

	// Store in cache and set current actionsData
	actionsDataMap[cacheKey] = newData
	actionsData = newData

	return nil
}

func listActions(c echo.Context) error {
	query := c.QueryParam("q")
	category := c.QueryParam("category")
	actionType := c.QueryParam("type")
	lang := c.QueryParam("lang")

	if err := loadActionsData(lang); err != nil {
		return c.JSON(http.StatusBadRequest, map[string]string{"error": err.Error()})
	}

	var summaries []ActionSummary

	for _, action := range actionsData.Actions {
		if matchesSearch(action, query, category, actionType) {
			summaries = append(summaries, ActionSummary{
				Name:        action.Name,
				Description: action.Description,
				Type:        string(action.Type),
				Categories:  action.Categories,
			})
		}
	}

	return c.JSON(http.StatusOK, summaries)
}

func getSegregatedActions(c echo.Context) error {
	query := c.QueryParam("q")
	lang := c.QueryParam("lang")

	if err := loadActionsData(lang); err != nil {
		return c.JSON(http.StatusBadRequest, map[string]string{"error": err.Error()})
	}

	segregated := SegregatedActions{
		ByCategory: make(map[string][]ActionSummary),
		ByType:     make(map[string][]ActionSummary),
	}

	for _, action := range actionsData.Actions {
		if matchesSearch(action, query, "", "") {
			summary := ActionSummary{
				Name:        action.Name,
				Description: action.Description,
				Type:        string(action.Type),
				Categories:  action.Categories,
			}

			if len(action.Categories) > 0 {
				for _, category := range action.Categories {
					segregated.ByCategory[category] = append(segregated.ByCategory[category], summary)
				}
			} else {
				segregated.ByCategory["Uncategorized"] = append(segregated.ByCategory["Uncategorized"], summary)
			}

			segregated.ByType[string(action.Type)] = append(segregated.ByType[string(action.Type)], summary)
		}
	}

	return c.JSON(http.StatusOK, segregated)
}

func matchesSearch(action Action, query, category, actionType string) bool {
	if category != "" && !containsCaseInsensitive(action.Categories, category) {
		return false
	}

	if actionType != "" && !strings.EqualFold(string(action.Type), actionType) {
		return false
	}

	if query == "" {
		return true
	}

	searchFields := []string{
		action.Name,
		action.Description,
		string(action.Type),
	}
	searchFields = append(searchFields, action.Categories...)

	for _, field := range searchFields {
		if partialMatch(field, query) {
			return true
		}
	}

	return false
}

func partialMatch(s, query string) bool {
	s = strings.ToLower(s)
	query = strings.ToLower(query)

	words := strings.Fields(s)
	queryWords := strings.Fields(query)

	for _, queryWord := range queryWords {
		matched := false
		for _, word := range words {
			if strings.HasPrefix(word, queryWord) {
				matched = true
				break
			}
		}
		if !matched {
			return false
		}
	}
	return true
}

func containsCaseInsensitive(slice []string, s string) bool {
	for _, item := range slice {
		if strings.EqualFold(item, s) {
			return true
		}
	}
	return false
}

func getActionDetails(c echo.Context) error {
	id := c.Param("id")
	lang := c.QueryParam("lang")

	if err := loadActionsData(lang); err != nil {
		return c.JSON(http.StatusBadRequest, map[string]string{"error": err.Error()})
	}

	for _, action := range actionsData.Actions {
		if action.Name == id {
			return c.JSON(http.StatusOK, action)
		}
	}

	return c.JSON(http.StatusNotFound, map[string]string{"error": "Action not found"})
}

func SetupActionRoutes(e *echo.Echo) {
	e.GET("/actions", listActions)
	e.GET("/actions/segregated", getSegregatedActions)
	e.GET("/actions/:id", getActionDetails)
}
