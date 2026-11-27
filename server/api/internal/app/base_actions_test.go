package app

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"sort"
	"strings"
	"testing"
)

var (
	japaneseText  = regexp.MustCompile(`[\p{Hiragana}\p{Katakana}\p{Han}]`)
	lowercaseText = regexp.MustCompile(`[a-z]`)
	// A definition's own text is never shown: the property that references it supplies
	// the form's label, and the i18n tooling cannot reach it anyway.
	definitionOwnText = regexp.MustCompile(`^/parameter/definitions/[^/]+/(title|description)$`)
)

// Strings that read the same in Japanese, so an untranslated value is correct.
var untranslatedInJapanese = map[string]bool{
	"Base64": true,
	"gml:id": true,
}

// Palette actions whose parameter text the i18n tooling cannot reach yet.
var untranslatableActions = map[string]string{
	// Its parameter block is a root-level oneOf (action-standard §3.4).
	"XML Fragmenter": "parameter block must be restructured before it can be translated",
}

// The i18n scaffold seeds every new key with its English text, and the schema check
// only confirms the generated files match their source, so an action can reach the
// palette with English in the Japanese UI. This fails until those values are replaced.
func TestBaseActionsAreTranslatedToJapanese(t *testing.T) {
	raw, err := os.ReadFile(filepath.Join("..", "..", "..", "..", "engine", "schema", "actions_ja.json"))
	if err != nil {
		t.Fatalf("read actions_ja.json: %v", err)
	}
	var schema struct {
		Actions []map[string]any `json:"actions"`
	}
	if err := json.Unmarshal(raw, &schema); err != nil {
		t.Fatalf("parse actions_ja.json: %v", err)
	}
	byName := make(map[string]map[string]any, len(schema.Actions))
	for _, action := range schema.Actions {
		name, _ := action["name"].(string)
		byName[name] = action
	}

	names := make([]string, 0, len(baseActions))
	for name := range baseActions {
		names = append(names, name)
	}
	sort.Strings(names)

	for _, name := range names {
		t.Run(name, func(t *testing.T) {
			if reason, ok := untranslatableActions[name]; ok {
				t.Skip(reason)
			}
			action, ok := byName[name]
			if !ok {
				t.Fatalf("not in actions_ja.json")
			}
			var english []string
			collectText(action, "", func(path, text string) {
				if definitionOwnText.MatchString(path) || japaneseText.MatchString(text) ||
					!lowercaseText.MatchString(text) || untranslatedInJapanese[text] {
					return
				}
				english = append(english, fmt.Sprintf("%s: %q", path, text))
			})
			if len(english) > 0 {
				t.Errorf("English text in the Japanese schema; translate it in engine/schema/i18n/actions/ja.json:\n  %s",
					strings.Join(english, "\n  "))
			}
		})
	}
}

// collectText calls visit with every title and description under node, keyed by JSON path.
func collectText(node any, path string, visit func(path, text string)) {
	switch n := node.(type) {
	case map[string]any:
		for key, value := range n {
			if text, ok := value.(string); ok && (key == "title" || key == "description") {
				visit(path+"/"+key, text)
				continue
			}
			collectText(value, path+"/"+key, visit)
		}
	case []any:
		for i, value := range n {
			collectText(value, fmt.Sprintf("%s/%d", path, i), visit)
		}
	}
}
