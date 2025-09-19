package user

import "strings"

type Theme string

const (
	ThemeDefault Theme = "default"
	ThemeLight   Theme = "light"
	ThemeDark    Theme = "dark"
)

func ThemeFrom(s string) Theme {
	switch strings.ToLower(s) {
	case "dark":
		return ThemeDark
	case "light":
		return ThemeLight
	}
	return ThemeDefault
}

func (t Theme) Ref() *Theme {
	return &t
}
func (t Theme) Valid() bool {
	switch t {
	case ThemeDark, ThemeLight, ThemeDefault:
		return true
	}
	return false
}
