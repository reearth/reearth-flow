# HTTP Caller Action

## Overview

The HTTP Caller action calls an HTTP or HTTPS endpoint for each feature and stores the response in feature attributes. It supports several authentication methods, request bodies including file uploads, retry with exponential backoff, rate limiting, and saving responses to files.

## When to Use

- **API Integration**: Fetch data from REST APIs to enrich features
- **Geocoding**: Convert addresses to coordinates using geocoding services
- **Validation**: Validate data against external services
- **File Downloads**: Download files referenced in feature attributes
- **Webhooks**: Send feature data to external systems

## Network Egress Control

Because workflows run server-side, outbound requests are restricted:

- Only `http` and `https` URLs are allowed.
- Requests to private, loopback, link-local (including cloud metadata), and
  other internal network addresses are blocked — whether the URL names the
  address directly, a hostname resolves to one, or a redirect points at one.
- Self-hosted deployments that need to reach services on private addresses can
  opt out by setting the environment variable
  `FLOW_RUNTIME_HTTP_ALLOW_PRIVATE_NETWORK=true` where the engine runs.
  The scheme restriction and the block on addresses that are never valid HTTP
  targets (unspecified, broadcast, multicast) always apply.

A blocked request routes the feature to the `rejected` port with the reason in
the `_http_error` attribute.

## Parameter Syntax

String-valued parameters marked "supports expressions" take a code object with
`type` (`string` for a literal, `flowExpr` for an expression) and `value`. An
expression is evaluated per feature and can use `attributes[...]` and
`variables[...]`.

## Basic Usage

### Simple GET Request

```yaml
- id: 550e8400-e29b-41d4-a716-446655440020
  name: Fetch-Weather
  type: action
  action: HTTP Caller
  with:
    url:
      type: flowExpr
      value: '"https://api.weather.example/current?lat=" + attributes["latitude"] + "&lon=" + attributes["longitude"]'
    method: GET
    response:
      responseBodyAttribute: _weather_data
```

### POST with JSON Body

```yaml
- id: 550e8400-e29b-41d4-a716-446655440021
  name: Geocode-Address
  type: action
  action: HTTP Caller
  with:
    url:
      type: string
      value: https://geocoding.example.com/geocode
    method: POST
    requestBody:
      type: text
      content:
        type: flowExpr
        value: |
          json.dumps({
            "address": attributes["address"],
            "city": attributes["city"]
          })
      contentType: "application/json"
    response:
      responseBodyAttribute: _geocoded_result
```

## Authentication Methods

Credential values support expressions, so secrets can come from workflow
variables instead of being hardcoded in the workflow file.

### Basic Authentication

```yaml
authentication:
  type: basic
  username:
    type: string
    value: my_username
  password:
    type: flowExpr
    value: variables["API_PASSWORD"]
```

### Bearer Token (OAuth 2.0)

```yaml
authentication:
  type: bearer
  token:
    type: flowExpr
    value: variables["API_TOKEN"]
```

### API Key in Header or Query Parameter

```yaml
authentication:
  type: apiKey
  keyName: "X-API-Key"
  keyValue:
    type: flowExpr
    value: variables["API_KEY"]
  location: header # or: query
```

## Request Configuration

### Custom Headers

```yaml
customHeaders:
  - name: "Accept"
    value:
      type: string
      value: "application/json"
  - name: "X-Custom-Header"
    value:
      type: flowExpr
      value: attributes["custom_value"]
```

### Query Parameters

```yaml
queryParameters:
  - name: "format"
    value:
      type: string
      value: "json"
  - name: "limit"
    value:
      type: string
      value: "10"
```

### Request Bodies

#### Form URL Encoded

```yaml
requestBody:
  type: formUrlEncoded
  fields:
    - name: "username"
      value:
        type: flowExpr
        value: attributes["username"]
```

#### Multipart Form Data (File Upload)

Multipart bodies cannot be combined with retry — the combination is rejected
when the workflow is built.

```yaml
requestBody:
  type: multipart
  parts:
    - type: text
      name: "description"
      value:
        type: string
        value: "Upload from workflow"
    - type: file
      name: "document"
      source:
        type: file
        path:
          type: flowExpr
          value: attributes["file_path"]
      filename: "document.pdf"
      contentType: "application/pdf"
```

#### Binary from Base64

```yaml
requestBody:
  type: binary
  source:
    type: base64
    data:
      type: flowExpr
      value: attributes["image_base64"]
  contentType: "image/png"
```

## Response Handling

Every successful response stores the status code, response headers, and — on
the `rejected` port — any error message, in fixed attributes:

| Attribute | Content |
| --- | --- |
| `_http_status_code` | HTTP status code (e.g. 200, 404) |
| `_headers` | Response headers as a map |
| `_http_error` | Error message, on rejected features only |

The response body destination is configurable.

### Store in Attribute (Default)

```yaml
response:
  responseBodyAttribute: _response_body
```

### Save to File

The path is relative to the job's output directory; absolute paths and path
traversal are rejected. The saved location is stored in the
`_response_file_path` attribute.

```yaml
response:
  responseHandling:
    type: file
    path:
      type: flowExpr
      value: '"downloads/" + attributes["id"] + ".json"'
```

### Response Encoding

When `responseEncoding` is not set, text or base64 storage is chosen from the
response's `Content-Type` header (disable with `autoDetectEncoding: false`;
the fallback is text). An explicit `responseEncoding` always wins.

```yaml
response:
  responseEncoding: text # UTF-8 text
  # responseEncoding: base64 # base64-encoded string, for binary data
```

### Limit Response Size

The download is stopped and the feature rejected as soon as the limit is
exceeded.

```yaml
response:
  maxResponseSize: 10485760 # 10MB
```

## Retry Configuration

Automatically retry failed requests with exponential backoff. `maxAttempts`
counts all attempts including the first, so `3` means one request plus up to
two retries; `1` disables retries. When `retryOnStatus` is omitted, all 5xx
status codes are retried. `Retry-After` response headers (seconds or HTTP-date
form) are honored by default, capped at `maxDelayMs`.

```yaml
retry:
  maxAttempts: 3
  initialDelayMs: 100
  backoffMultiplier: 2.0
  maxDelayMs: 10000
  retryOnStatus: [429, 503, 504]
  honorRetryAfter: true
```

## Rate Limiting

The action makes one request per feature, so a large feature stream without a
rate limit hammers the target API. The limiter is shared across worker
threads.

```yaml
rateLimit:
  requests: 100
  intervalMs: 60000 # 100 requests per minute
  timing: distributed # or: burst
```

- **burst**: send requests immediately until the limit, then pause until the next interval
- **distributed**: space requests evenly throughout the interval

## Timeouts

```yaml
timeouts:
  connectionTimeout: 30 # seconds to establish a connection (default: 60)
  transferTimeout: 300 # seconds for the entire request (default: 90); raise for large downloads
```

## HTTP Options

```yaml
httpOptions:
  verifySsl: true # default; disable only for self-signed certificates
  followRedirects: true # default
  maxRedirects: 10 # default
  userAgent: "MyApp/1.0"
```

## Output Ports

- **features**: features whose request completed (any status code, including 4xx/5xx)
- **rejected**: features whose request could not be completed (invalid or blocked URL, network failure after retries, oversized response, file save failure)

## Architecture Notes

- The reqwest client is lazy-initialized once and shared across processor
  clones for connection pooling (`Arc<OnceCell>`).
- All expressions are compiled at build time, not per feature.
- Egress control is layered: URL validation per feature, a filtering DNS
  resolver on the client (which also covers redirects and DNS rebinding), and
  a per-hop redirect policy for literal-IP targets. See `egress.rs`.
- Response file writes are restricted to the job's sandbox root, mirroring the
  sink-output chokepoint.
