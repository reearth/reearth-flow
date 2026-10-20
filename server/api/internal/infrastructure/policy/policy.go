// Package policy implements gateway.PolicyChecker. The HTTP checker calls the
// shared Re:Earth policy-check endpoint, which answers from the workspace's plan;
// the permissive checker allows everything, for deployments without plans.
package policy

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"time"

	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearthx/rerror"
)

type permissive struct{}

func NewPermissive() gateway.PolicyChecker {
	return permissive{}
}

func (permissive) CheckPolicy(_ context.Context, req gateway.PolicyCheckRequest) (*gateway.PolicyCheckResponse, error) {
	return &gateway.PolicyCheckResponse{
		Allowed:      true,
		CheckType:    req.CheckType,
		CurrentLimit: gateway.PolicyLimitUnlimited,
		Value:        req.Value,
	}, nil
}

type httpChecker struct {
	client   *http.Client
	endpoint string
	token    string
}

func NewHTTP(endpoint, token string, timeout time.Duration) gateway.PolicyChecker {
	return &httpChecker{
		client:   &http.Client{Timeout: timeout},
		endpoint: endpoint,
		token:    token,
	}
}

func (h *httpChecker) CheckPolicy(ctx context.Context, req gateway.PolicyCheckRequest) (*gateway.PolicyCheckResponse, error) {
	body, err := json.Marshal(req)
	if err != nil {
		return nil, rerror.ErrInternalByWithContext(ctx, fmt.Errorf("policy: marshal request: %w", err))
	}

	httpReq, err := http.NewRequestWithContext(ctx, http.MethodPost, h.endpoint, bytes.NewReader(body))
	if err != nil {
		return nil, rerror.ErrInternalByWithContext(ctx, fmt.Errorf("policy: build request: %w", err))
	}
	httpReq.Header.Set("Content-Type", "application/json")
	if h.token != "" {
		httpReq.Header.Set("Authorization", "Bearer "+h.token)
	}

	resp, err := h.client.Do(httpReq)
	if err != nil {
		return nil, rerror.ErrInternalByWithContext(ctx, fmt.Errorf("policy: request failed: %w", err))
	}
	defer func() { _ = resp.Body.Close() }()

	if resp.StatusCode != http.StatusOK {
		return nil, rerror.ErrInternalByWithContext(ctx, fmt.Errorf("policy: check returned status %d", resp.StatusCode))
	}

	var res gateway.PolicyCheckResponse
	if err := json.NewDecoder(resp.Body).Decode(&res); err != nil {
		return nil, rerror.ErrInternalByWithContext(ctx, fmt.Errorf("policy: decode response: %w", err))
	}
	return &res, nil
}
