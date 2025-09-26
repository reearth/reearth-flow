package gqlmodel

import (
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/url"
	"strconv"

	"github.com/99designs/gqlgen/graphql"
	"github.com/reearth/reearthx/usecasex"
	"golang.org/x/text/language"
)

func MarshalURL(t url.URL) graphql.Marshaler {
	return graphql.WriterFunc(func(w io.Writer) {
		_, _ = io.WriteString(w, strconv.Quote(t.String()))
	})
}

func UnmarshalURL(v interface{}) (url.URL, error) {
	if tmpStr, ok := v.(string); ok {
		u, err := url.Parse(tmpStr)
		if u != nil {
			return *u, err
		}
		return url.URL{}, err
	}
	return url.URL{}, errors.New("invalid URL")
}

func MarshalLang(t language.Tag) graphql.Marshaler {
	return graphql.WriterFunc(func(w io.Writer) {
		_, _ = io.WriteString(w, strconv.Quote(t.String()))
	})
}

func UnmarshalLang(v interface{}) (language.Tag, error) {
	if tmpStr, ok := v.(string); ok {
		if tmpStr == "" {
			return language.Tag{}, nil
		}
		l, err := language.Parse(tmpStr)
		if err != nil {
			return language.Tag{}, err
		}
		return l, nil
	}
	return language.Tag{}, errors.New("invalid lang")
}

func MarshalCursor(t usecasex.Cursor) graphql.Marshaler {
	return graphql.WriterFunc(func(w io.Writer) {
		_, _ = io.WriteString(w, strconv.Quote(string(t)))
	})
}

func UnmarshalCursor(v interface{}) (usecasex.Cursor, error) {
	if tmpStr, ok := v.(string); ok {
		return usecasex.Cursor(tmpStr), nil
	}
	return usecasex.Cursor(""), errors.New("invalid cursor")
}

func MarshalMap(val map[string]string) graphql.Marshaler {
	return graphql.WriterFunc(func(w io.Writer) {
		_ = json.NewEncoder(w).Encode(val)
	})
}

func UnmarshalMap(v interface{}) (map[string]string, error) {
	if m, ok := v.(map[string]string); ok {
		return m, nil
	}
	return nil, fmt.Errorf("%T is not a map", v)
}

type Bytes []byte

func MarshalBytes(b Bytes) graphql.Marshaler {
	return graphql.WriterFunc(func(w io.Writer) {
		nums := make([]int, len(b))
		for i, v := range b {
			nums[i] = int(v)
		}
		_ = json.NewEncoder(w).Encode(nums)
	})
}

func UnmarshalBytes(v interface{}) (Bytes, error) {
	if bytes, ok := v.([]byte); ok {
		return Bytes(bytes), nil
	}

	if arr, ok := v.([]interface{}); ok {
		bytes := make([]byte, len(arr))
		for i, item := range arr {
			if num, ok := item.(float64); ok {
				bytes[i] = byte(num)
			} else {
				return nil, fmt.Errorf("array element at index %d is not a number", i)
			}
		}
		return Bytes(bytes), nil
	}

	return nil, errors.New("Bytes must be a byte array or number array (Uint8Array)")
}
