package app

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"strings"
	"sync"
	"time"

	"github.com/labstack/echo/v4"
)

type ActionType string

const (
	ActionTypeProcessor ActionType = "processor"
	ActionTypeSource    ActionType = "source"
	ActionTypeSink      ActionType = "sink"
)

type Action struct {
	Name        string                 `json:"name"`
	Type        ActionType             `json:"type"`
	Description string                 `json:"description"`
	Parameter   map[string]interface{} `json:"parameter"`
	Builtin     bool                   `json:"builtin"`
	InputPorts  []string               `json:"inputPorts"`
	OutputPorts []string               `json:"outputPorts"`
	Categories  []string               `json:"categories"`
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

const defaultActionsBaseURL = "https://raw.githubusercontent.com/reearth/reearth-flow/main/engine/schema/"

type actionCache struct {
	mu      sync.RWMutex
	data    map[string]ActionsData
	client  *http.Client
	baseURL string
}

func newActionCache(client *http.Client, baseURL string) *actionCache {
	if client == nil {
		client = &http.Client{Timeout: 10 * time.Second}
	}
	if baseURL == "" {
		baseURL = defaultActionsBaseURL
	}
	baseURL = strings.TrimRight(baseURL, "/") + "/"

	return &actionCache{
		data:    make(map[string]ActionsData),
		client:  client,
		baseURL: baseURL,
	}
}

func (c *actionCache) load(ctx context.Context, lang string) (ActionsData, error) {
	if lang != "" && !supportedLangs[lang] {
		return ActionsData{}, fmt.Errorf("unsupported language: %s", lang)
	}

	cacheKey := lang

	c.mu.RLock()
	if data, exists := c.data[cacheKey]; exists {
		c.mu.RUnlock()
		return data, nil
	}
	c.mu.RUnlock()

	c.mu.Lock()
	defer c.mu.Unlock()
	if data, exists := c.data[cacheKey]; exists {
		return data, nil
	}

	data, err := c.fetch(ctx, lang)
	if err != nil {
		return ActionsData{}, err
	}

	c.data[cacheKey] = data
	return data, nil
}

func (c *actionCache) fetch(ctx context.Context, lang string) (ActionsData, error) {
	filename := "actions.json"
	if lang != "" {
		filename = fmt.Sprintf("actions_%s.json", lang)
	}

	url := c.baseURL + filename
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, url, nil)
	if err != nil {
		return ActionsData{}, err
	}

	resp, err := c.client.Do(req)
	if err != nil {
		return ActionsData{}, err
	}
	defer func() {
		if cerr := resp.Body.Close(); cerr != nil {
			fmt.Println("Error closing response body:", cerr)
		}
	}()

	if resp.StatusCode != http.StatusOK {
		return ActionsData{}, fmt.Errorf("failed to load actions: status %d", resp.StatusCode)
	}

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return ActionsData{}, err
	}

	var newData ActionsData
	if err := json.Unmarshal(body, &newData); err != nil {
		return ActionsData{}, err
	}

	if err := newData.Validate(); err != nil {
		return ActionsData{}, err
	}

	return newData, nil
}

func (c *actionCache) set(lang string, data ActionsData) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.data[lang] = data
}

func (c *actionCache) reset() {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.data = make(map[string]ActionsData)
}

func (c *actionCache) get(lang string) (ActionsData, bool) {
	c.mu.RLock()
	defer c.mu.RUnlock()
	data, ok := c.data[lang]
	return data, ok
}

var (
	actionsDataStore = newActionCache(nil, "")
	supportedLangs   = map[string]bool{
		"en": true,
		"es": true,
		"fr": true,
		"ja": true,
		"zh": true,
	}
)

func loadActionsData(ctx context.Context, lang string) (ActionsData, error) {
	return actionsDataStore.load(ctx, lang)
}

func listActions(c echo.Context) error {
	query := c.QueryParam("q")
	category := c.QueryParam("category")
	actionType := c.QueryParam("type")
	lang := c.QueryParam("lang")

	data, err := loadActionsData(c.Request().Context(), lang)
	if err != nil {
		return c.JSON(http.StatusBadRequest, map[string]string{"error": err.Error()})
	}

	var summaries []ActionSummary

	for _, action := range data.Actions {
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

	data, err := loadActionsData(c.Request().Context(), lang)
	if err != nil {
		return c.JSON(http.StatusBadRequest, map[string]string{"error": err.Error()})
	}

	segregated := SegregatedActions{
		ByCategory: make(map[string][]ActionSummary),
		ByType:     make(map[string][]ActionSummary),
	}

	for _, action := range data.Actions {
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

	data, err := loadActionsData(c.Request().Context(), lang)
	if err != nil {
		return c.JSON(http.StatusBadRequest, map[string]string{"error": err.Error()})
	}

	for _, action := range data.Actions {
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
