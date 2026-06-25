// Package logging builds the service's slog.Logger from a configured level and
// format, so production can emit structured JSON at a chosen verbosity while
// local development stays human-readable.
package logging

import (
	"io"
	"log/slog"
	"strings"
)

// New builds a slog.Logger writing to w.
//
// level is one of debug/info/warn/error (case-insensitive); any other value
// falls back to info. format is "json" or "text" (case-insensitive); any other
// value falls back to text.
func New(level, format string, w io.Writer) *slog.Logger {
	opts := &slog.HandlerOptions{Level: parseLevel(level)}
	var h slog.Handler
	if strings.EqualFold(strings.TrimSpace(format), "json") {
		h = slog.NewJSONHandler(w, opts)
	} else {
		h = slog.NewTextHandler(w, opts)
	}
	return slog.New(h)
}

// parseLevel maps a level name onto a slog.Level, defaulting to info.
func parseLevel(s string) slog.Level {
	switch strings.ToLower(strings.TrimSpace(s)) {
	case "debug":
		return slog.LevelDebug
	case "warn", "warning":
		return slog.LevelWarn
	case "error":
		return slog.LevelError
	default:
		return slog.LevelInfo
	}
}
