package logging

import (
	"bytes"
	"strings"
	"testing"
)

func TestNewRespectsLevel(t *testing.T) {
	var buf bytes.Buffer
	log := New("warn", "text", &buf)
	log.Info("info-line")
	log.Warn("warn-line")
	out := buf.String()
	if strings.Contains(out, "info-line") {
		t.Errorf("info logged at warn level:\n%s", out)
	}
	if !strings.Contains(out, "warn-line") {
		t.Errorf("warn not logged at warn level:\n%s", out)
	}
}

func TestNewDebugLevelEmitsDebug(t *testing.T) {
	var buf bytes.Buffer
	log := New("debug", "text", &buf)
	log.Debug("debug-line")
	if !strings.Contains(buf.String(), "debug-line") {
		t.Errorf("debug not logged at debug level:\n%s", buf.String())
	}
}

func TestNewJSONFormat(t *testing.T) {
	var buf bytes.Buffer
	log := New("info", "json", &buf)
	log.Info("hello", "k", "v")
	out := strings.TrimSpace(buf.String())
	if !strings.HasPrefix(out, "{") || !strings.Contains(out, `"msg":"hello"`) {
		t.Errorf("not JSON:\n%s", out)
	}
}

func TestNewTextFormat(t *testing.T) {
	var buf bytes.Buffer
	log := New("info", "text", &buf)
	log.Info("hello")
	out := strings.TrimSpace(buf.String())
	if strings.HasPrefix(out, "{") {
		t.Errorf("want text, got JSON:\n%s", out)
	}
	if !strings.Contains(out, "msg=hello") {
		t.Errorf("text missing msg=hello:\n%s", out)
	}
}

func TestNewUnknownLevelDefaultsToInfo(t *testing.T) {
	var buf bytes.Buffer
	log := New("bogus", "text", &buf)
	log.Debug("debug-line")
	log.Info("info-line")
	out := buf.String()
	if strings.Contains(out, "debug-line") {
		t.Errorf("debug emitted at default (info) level:\n%s", out)
	}
	if !strings.Contains(out, "info-line") {
		t.Errorf("info not emitted at default (info) level:\n%s", out)
	}
}

func TestNewUnknownFormatDefaultsToText(t *testing.T) {
	var buf bytes.Buffer
	log := New("info", "bogus", &buf)
	log.Info("hello")
	if strings.HasPrefix(strings.TrimSpace(buf.String()), "{") {
		t.Errorf("unknown format should default to text, got JSON:\n%s", buf.String())
	}
}
