# HTTPCaller Action

## Overview

The HTTPCaller action makes HTTP/HTTPS requests and enriches features with response data. It supports a wide range of HTTP features including various authentication methods, retry logic, rate limiting, and flexible response handling.

## When to Use

- **API Integration**: Fetch data from REST APIs to enrich features
- **Geocoding**: Convert addresses to coordinates using geocoding services
- **Validation**: Validate data against external services
- **File Downloads**: Download files referenced in feature attributes
- **Webhooks**: Send feature data to external systems
- **Data Enrichment**: Augment features with data from external sources

## Basic Usage

### Simple GET Request

```yaml
- id: fetch_weather
  type: HTTPCaller
  with:
    url: "https://api.weather.com/current?lat=${feature.latitude}&lon=${feature.longitude}"
    method: GET
    response:
      responseBodyAttribute: _weather_data
```

### POST with JSON Body

```yaml
- id: geocode_address
  type: HTTPCaller
  with:
    url: "https://geocoding.api.com/geocode"
    method: POST
    requestBody:
      type: text
      content: |
        {
          "address": "${feature.address}",
          "city": "${feature.city}"
        }
      contentType: "application/json"
    response:
      responseBodyAttribute: _geocoded_result
```

## Authentication Methods

### Basic Authentication

```yaml
authentication:
  type: basic
  username: "my_username"
  password: "my_password"
```

### Bearer Token (OAuth 2.0)

```yaml
authentication:
  type: bearer
  token: "${env.API_TOKEN}"
```

### API Key in Header

```yaml
authentication:
  type: apiKey
  keyName: "X-API-Key"
  keyValue: "${env.API_KEY}"
  location: header
```

### API Key in Query Parameter

```yaml
authentication:
  type: apiKey
  keyName: "api_key"
  keyValue: "${env.API_KEY}"
  location: query
```

## Request Configuration

### Custom Headers

```yaml
customHeaders:
  - name: "Accept"
    value: "application/json"
  - name: "X-Custom-Header"
    value: "${feature.custom_value}"
```

### Query Parameters

```yaml
queryParameters:
  - name: "format"
    value: "json"
  - name: "limit"
    value: "10"
```

### Request Bodies

#### Form URL Encoded

```yaml
requestBody:
  type: formUrlEncoded
  fields:
    - name: "username"
      value: "${feature.username}"
    - name: "email"
      value: "${feature.email}"
```

#### Multipart Form Data (File Upload)

```yaml
requestBody:
  type: multipart
  parts:
    - type: text
      name: "description"
      value: "Upload from workflow"
    - type: file
      name: "document"
      source:
        type: file
        path: "${feature.file_path}"
      filename: "document.pdf"
      contentType: "application/pdf"
```

#### Binary from Base64

```yaml
requestBody:
  type: binary
  source:
    type: base64
    data: "${feature.image_base64}"
  contentType: "image/png"
```

## Response Handling

### Store in Attribute (Default)

```yaml
response:
  responseHandling:
    type: attribute
  responseBodyAttribute: _response_body
  statusCodeAttribute: _http_status_code
  headersAttribute: _headers
  errorAttribute: _http_error
```

### Save to File

```yaml
response:
  responseHandling:
    type: file
    path: "/tmp/downloads/${feature.id}.json"
    storePathInAttribute: true
    pathAttribute: _downloaded_file_path
```

### Response Encoding

```yaml
response:
  responseEncoding: text        # UTF-8 text (default)
  # responseEncoding: base64    # Base64-encoded string
  # responseEncoding: binary    # Raw binary data
```

## Retry Configuration

Automatically retry failed requests with exponential backoff:

```yaml
retry:
  maxAttempts: 3                    # Maximum number of retry attempts
  initialDelayMs: 100               # Initial delay before first retry
  backoffMultiplier: 2.0            # Exponential backoff multiplier
  maxDelayMs: 10000                 # Maximum delay between retries
  retryOnStatus: [429, 503, 504]    # HTTP status codes to retry
  honorRetryAfter: true             # Respect Retry-After header
```

### Example: Robust API Integration

```yaml
- id: fetch_with_retry
  type: HTTPCaller
  with:
    url: "https://api.example.com/data"
    retry:
      maxAttempts: 5
      initialDelayMs: 200
      backoffMultiplier: 2.0
      maxDelayMs: 30000
      retryOnStatus: [429, 500, 502, 503, 504]
```

## Rate Limiting

Control request frequency to avoid overwhelming APIs:

```yaml
rateLimit:
  requests: 10          # Maximum requests
  intervalMs: 1000      # Within this interval (1 second)
  timing: burst         # or "distributed"
```

### Timing Strategies

- **burst**: Send all requests immediately, then pause until next interval
- **distributed**: Evenly distribute requests throughout the interval

### Example: Rate-Limited API Access

```yaml
- id: fetch_with_rate_limit
  type: HTTPCaller
  with:
    url: "https://api.example.com/data/${feature.id}"
    rateLimit:
      requests: 100
      intervalMs: 60000  # 100 requests per minute
      timing: distributed
```

## Timeouts

Configure connection and transfer timeouts:

```yaml
timeouts:
  connectionTimeout: 30     # Seconds to establish connection
  transferTimeout: 60       # Seconds to complete entire request
```

## HTTP Options

Configure HTTP client behavior:

```yaml
httpOptions:
  verifySsl: true           # Verify SSL certificates (default: true)
                            # Set to false for self-signed certificates (not recommended for production)
  followRedirects: true     # Automatically follow redirects (default)
  maxRedirects: 10          # Maximum number of redirects to follow
  userAgent: "MyApp/1.0"    # Custom User-Agent header
```

## Observability

Track additional metrics about HTTP requests:

```yaml
observability:
  trackDuration: true                    # Track request duration
  durationAttribute: "_request_duration_ms"

  trackFinalUrl: true                    # Track final URL after redirects
  finalUrlAttribute: "_final_url"

  trackRetryCount: true                  # Track number of retries
  retryCountAttribute: "_retry_count"

  trackBytes: true                       # Track response size
  bytesAttribute: "_response_bytes"
```

## Complete Examples

### Geocoding API with Retry and Rate Limiting

```yaml
- id: geocode_addresses
  type: HTTPCaller
  with:
    url: "https://geocoding.api.com/geocode"
    method: POST

    authentication:
      type: apiKey
      keyName: "X-API-Key"
      keyValue: "${env.GEOCODING_API_KEY}"
      location: header

    requestBody:
      type: text
      content: |
        {
          "address": "${feature.street_address}",
          "city": "${feature.city}",
          "country": "${feature.country}"
        }
      contentType: "application/json"

    retry:
      maxAttempts: 3
      initialDelayMs: 100
      backoffMultiplier: 2.0
      retryOnStatus: [429, 503]

    rateLimit:
      requests: 50
      intervalMs: 1000
      timing: distributed

    response:
      responseBodyAttribute: _geocoded_data
      statusCodeAttribute: _geocode_status
```

### File Download

```yaml
- id: download_image
  type: HTTPCaller
  with:
    url: "${feature.image_url}"
    method: GET

    response:
      responseHandling:
        type: file
        path: "/tmp/images/${feature.id}.jpg"
        storePathInAttribute: true
        pathAttribute: _image_file_path

    timeouts:
      connectionTimeout: 30
      transferTimeout: 300

    observability:
      trackDuration: true
      trackBytes: true
```

### REST API Integration with OAuth

```yaml
- id: fetch_user_data
  type: HTTPCaller
  with:
    url: "https://api.example.com/v1/users/${feature.user_id}"
    method: GET

    authentication:
      type: bearer
      token: "${env.OAUTH_ACCESS_TOKEN}"

    customHeaders:
      - name: "Accept"
        value: "application/json"
      - name: "X-Request-ID"
        value: "${feature.request_id}"

    retry:
      maxAttempts: 3
      honorRetryAfter: true

    response:
      maxResponseSize: 1048576  # 1MB limit
      responseBodyAttribute: _user_data
```

### Webhook Notification

```yaml
- id: send_webhook
  type: HTTPCaller
  with:
    url: "https://webhooks.example.com/notify"
    method: POST

    customHeaders:
      - name: "Content-Type"
        value: "application/json"
      - name: "X-Webhook-Secret"
        value: "${env.WEBHOOK_SECRET}"

    requestBody:
      type: text
      content: |
        {
          "event": "feature_processed",
          "feature_id": "${feature.id}",
          "timestamp": "${datetime::now()}",
          "data": ${feature.data}
        }

    retry:
      maxAttempts: 5
      initialDelayMs: 1000
```

## Best Practices

### Use Environment Variables for Secrets

Store API keys and tokens in environment variables, not in workflow files:

```yaml
authentication:
  type: bearer
  token: "${env.API_TOKEN}"  # Good
  # token: "sk-abc123..."     # Bad - hardcoded secret
```

### Set Appropriate Timeouts

```yaml
timeouts:
  connectionTimeout: 5      # Fast APIs
  transferTimeout: 10

timeouts:
  connectionTimeout: 30     # Slow APIs or file downloads
  transferTimeout: 120      # Use 300 for large file downloads
```

### Use Retry for Resilience

Always configure retry for production workflows:

```yaml
retry:
  maxAttempts: 3
  retryOnStatus: [429, 500, 502, 503, 504]
  honorRetryAfter: true
```

### Rate Limit to Avoid Throttling

Check your API's rate limits and configure accordingly:

```yaml
rateLimit:
  requests: 100
  intervalMs: 60000  # Match your API's limits
  timing: distributed
```

### Monitor with Observability

Enable observability to track performance and debug issues:

```yaml
observability:
  trackDuration: true
  trackRetryCount: true
  trackBytes: true
```

### Limit Response Sizes

Protect your workflow from large responses:

```yaml
response:
  maxResponseSize: 10485760  # 10MB limit
```

## Troubleshooting

### Request Timeouts

- Increase timeouts with `timeouts: { connectionTimeout: 60, transferTimeout: 120 }`
- Check network connectivity
- Verify the remote server is responsive

### Authentication Failures (401/403)

- Verify credentials are correct
- Check if token/key has expired
- Ensure API key location (header vs. query) is correct

### Rate Limiting (429)

- Configure retry with `honorRetryAfter: true`
- Reduce request frequency with `rateLimit`
- Contact API provider for higher limits

### SSL Certificate Errors

- Ensure `httpOptions: { verifySsl: true }` (default)
- For self-signed certificates (development only): `httpOptions: { verifySsl: false }`
- Check if certificates are valid and not expired

### Large Response Bodies

- Set `response: { maxResponseSize: 10485760 }` to limit memory usage
- Use `response: { responseHandling: { type: file, path: ... } }` for large files
- Consider streaming APIs if available

## Expression Support

Most string parameters support Rhai expressions:

```yaml
url: "https://api.example.com/v1/users/${feature.id}"
queryParameters:
  - name: "timestamp"
    value: "${datetime::now()}"
  - name: "hash"
    value: "${str::sha256(feature.data)}"
```

See the [Expression Documentation](../../../../docs/expression-math-functions.md) for available functions.

## Output Ports

- **default**: Features with successful HTTP responses
- **rejected**: Features where HTTP requests failed

## Default Attribute Names

- `_response_body`: The response body content
- `_http_status_code`: The HTTP status code (e.g., 200, 404)
- `_headers`: Response headers as a JSON object
- `_http_error`: Error message (if request failed)

You can customize these with:
```yaml
response:
  responseBodyAttribute: custom_response
  statusCodeAttribute: custom_status
  headersAttribute: custom_headers
  errorAttribute: custom_error
```
