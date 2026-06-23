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
	"github.com/reearth/reearthx/log"
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
	Tags        []string               `json:"tags"`
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
	Tags        []string `json:"tags"`
}

type SegregatedActions struct {
	ByCategory map[string][]ActionSummary `json:"byCategory"`
	ByType     map[string][]ActionSummary `json:"byType"`
}

type loadError struct {
	err    error
	status int
}

func (e *loadError) Error() string { return e.err.Error() }
func (e *loadError) Unwrap() error { return e.err }

var (
	actionsDataMap = make(map[string]ActionsData)
	mutex          sync.RWMutex
	httpClient     = &http.Client{Timeout: 10 * time.Second}
	supportedLangs = map[string]bool{
		"en": true,
		"es": true,
		"fr": true,
		"ja": true,
		"zh": true,
	}
)

// actionsReader reads a schema file (e.g. "actions_en.json") from the env bucket's
// "actions/" prefix. gateway.File satisfies it via ReadActions.
type actionsReader interface {
	ReadActions(context.Context, string) (io.ReadCloser, error)
}

var (
	// actionsRepo, when set, is the bucket-backed schema source. nil → GitHub.
	actionsRepo actionsReader
	// actionsFallbackBaseURL is used when actionsRepo is unset or a read fails
	// (local dev / cold bucket). Overridable in tests.
	actionsFallbackBaseURL = "https://raw.githubusercontent.com/reearth/reearth-flow/main/engine/schema/"
)

func loadActionsData(lang string) (ActionsData, error) {
	if lang != "" && !supportedLangs[lang] {
		return ActionsData{}, &loadError{err: fmt.Errorf("unsupported language: %s", lang), status: http.StatusBadRequest}
	}

	cacheKey := lang

	// Try to get from cache first using read lock
	mutex.RLock()
	if data, exists := actionsDataMap[cacheKey]; exists {
		mutex.RUnlock()
		return data, nil
	}
	mutex.RUnlock()

	filename := "actions.json"
	if lang != "" {
		filename = fmt.Sprintf("actions_%s.json", lang)
	}

	body, err := fetchActionsFile(filename)
	if err != nil {
		return ActionsData{}, err
	}

	var newData ActionsData
	if err := json.Unmarshal(body, &newData); err != nil {
		return ActionsData{}, &loadError{err: err, status: http.StatusInternalServerError}
	}

	if err := newData.Validate(); err != nil {
		return ActionsData{}, &loadError{err: err, status: http.StatusInternalServerError}
	}

	// Store in cache under write lock; double-check in case a concurrent
	// goroutine already populated the same key while we were fetching.
	mutex.Lock()
	defer mutex.Unlock()
	if data, exists := actionsDataMap[cacheKey]; exists {
		return data, nil
	}
	actionsDataMap[cacheKey] = newData
	return newData, nil
}

// fetchActionsFile returns the bytes of a schema file, preferring the bucket-backed
// actionsRepo and falling back to GitHub (actionsFallbackBaseURL).
func fetchActionsFile(filename string) ([]byte, error) {
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	if actionsRepo != nil {
		rc, err := actionsRepo.ReadActions(ctx, filename)
		if err == nil {
			defer func() {
				if cerr := rc.Close(); cerr != nil {
					log.Warnf("actions: error closing reader: %v", cerr)
				}
			}()
			body, rerr := io.ReadAll(rc)
			if rerr != nil {
				return nil, &loadError{err: rerr, status: http.StatusInternalServerError}
			}
			return body, nil
		}
		log.Warnf("actions: bucket read failed for %s, falling back to GitHub: %v", filename, err)
	}

	url := actionsFallbackBaseURL + filename
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, url, nil)
	if err != nil {
		return nil, &loadError{err: err, status: http.StatusBadGateway}
	}
	resp, err := httpClient.Do(req)
	if err != nil {
		return nil, &loadError{err: err, status: http.StatusBadGateway}
	}
	defer func() {
		if cerr := resp.Body.Close(); cerr != nil {
			log.Warnf("actions: error closing response body: %v", cerr)
		}
	}()
	if resp.StatusCode != http.StatusOK {
		return nil, &loadError{err: fmt.Errorf("unexpected status %d fetching %s", resp.StatusCode, url), status: http.StatusBadGateway}
	}
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, &loadError{err: err, status: http.StatusInternalServerError}
	}
	return body, nil
}

// listActions godoc
// @Summary      List actions
// @Description  Get a list of available workflow actions with optional filtering
// @Tags         actions
// @Produce      json
// @Param        q         query     string  false  "Search query"
// @Param        category  query     string  false  "Filter by category"
// @Param        type      query     string  false  "Filter by action type (processor, source, sink)"
// @Param        lang      query     string  false  "Language code (en, es, fr, ja, zh)"
// @Success      200       {array}   ActionSummary  "List of actions"
// @Failure      400       {object}  object         "Invalid language"
// @Failure      500       {object}  object         "Internal server error"
// @Failure      502       {object}  object         "Upstream fetch error"
// @Router       /actions [get]
func listActions(c echo.Context) error {
	query := c.QueryParam("q")
	category := c.QueryParam("category")
	actionType := c.QueryParam("type")
	lang := c.QueryParam("lang")

	data, err := loadActionsData(lang)
	if err != nil {
		status := http.StatusInternalServerError
		var le *loadError
		if errors.As(err, &le) {
			status = le.status
		}
		return c.JSON(status, map[string]string{"error": err.Error()})
	}

	var summaries []ActionSummary

	for _, action := range data.Actions {
		if !baseActions[action.Name] {
			continue
		}
		if matchesSearch(action, query, category, actionType) {
			summaries = append(summaries, ActionSummary{
				Name:        action.Name,
				Description: action.Description,
				Type:        string(action.Type),
				Categories:  action.Categories,
				Tags:        action.Tags,
			})
		}
	}

	return c.JSON(http.StatusOK, summaries)
}

// getSegregatedActions godoc
// @Summary      Get segregated actions
// @Description  Get actions organized by category and type
// @Tags         actions
// @Produce      json
// @Param        q     query     string  false  "Search query"
// @Param        lang  query     string  false  "Language code (en, es, fr, ja, zh)"
// @Success      200   {object}  SegregatedActions  "Actions segregated by category and type"
// @Failure      400   {object}  object             "Invalid language"
// @Failure      500   {object}  object             "Internal server error"
// @Failure      502   {object}  object             "Upstream fetch error"
// @Router       /actions/segregated [get]
func getSegregatedActions(c echo.Context) error {
	query := c.QueryParam("q")
	lang := c.QueryParam("lang")

	data, err := loadActionsData(lang)
	if err != nil {
		status := http.StatusInternalServerError
		var le *loadError
		if errors.As(err, &le) {
			status = le.status
		}
		return c.JSON(status, map[string]string{"error": err.Error()})
	}

	segregated := SegregatedActions{
		ByCategory: make(map[string][]ActionSummary),
		ByType:     make(map[string][]ActionSummary),
	}

	for _, action := range data.Actions {
		if !baseActions[action.Name] {
			continue
		}
		if matchesSearch(action, query, "", "") {
			summary := ActionSummary{
				Name:        action.Name,
				Description: action.Description,
				Type:        string(action.Type),
				Categories:  action.Categories,
				Tags:        action.Tags,
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
	searchFields = append(searchFields, action.Tags...)

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

// getActionDetails godoc
// @Summary      Get action details
// @Description  Get detailed information about a specific action
// @Tags         actions
// @Produce      json
// @Param        id    path      string  true   "Action ID/Name"
// @Param        lang  query     string  false  "Language code (en, es, fr, ja, zh)"
// @Success      200   {object}  Action  "Action details"
// @Failure      400   {object}  object  "Invalid language"
// @Failure      404   {object}  object  "Action not found"
// @Failure      500   {object}  object  "Internal server error"
// @Failure      502   {object}  object  "Upstream fetch error"
// @Router       /actions/{id} [get]
func getActionDetails(c echo.Context) error {
	id := c.Param("id")
	lang := c.QueryParam("lang")

	data, err := loadActionsData(lang)
	if err != nil {
		status := http.StatusInternalServerError
		var le *loadError
		if errors.As(err, &le) {
			status = le.status
		}
		return c.JSON(status, map[string]string{"error": err.Error()})
	}

	for _, action := range data.Actions {
		if !baseActions[action.Name] {
			continue
		}
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
