# What?

Implement an HTTPCaller processor action that makes HTTP/HTTPS requests based on feature data and enriches features with response information. 

# Why?

We need implement it for the UseCase workflow and which is part of the Plateau 5 deliverables
[HTTPCaller](https://www.notion.so/HTTPCaller-26116e0fb1658042aef8dae1ae68c3ef?pvs=21) 

# How?

**Phase 1: Core MVP**

**Module Setup**

- [ ]  Create `runtime/action-processor/src/http/` module (`mod.rs`, `errors.rs`, `caller.rs`, `mapping.rs`)
- [ ]  Register module in `lib.rs` and `mapping.rs`
- [ ]  Add i18n entries for all languages (en, ja, zh, fr, es)

**Core Parameters**

- [ ]  Request URL (expression-based, can reference feature attributes)
- [ ]  Custom headers (name/value pairs with expression support)
- [ ]  HTTP method (GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS)
- [ ]  Query parameters (name/value pairs with expression support)
- [ ]  Request body (expression for POST/PUT/PATCH)
- [ ]  Content-Type header
- [ ]  Response body attribute name (default: `_response_body`)
- [ ]  Status code attribute name (default: `_http_status_code`)
- [ ]  Headers list attribute name (default: `_headers`)
- [ ]  Error attribute name (default: `_http_error`)
- [ ]  Connection timeout (default: 60s)
- [ ]  Transfer timeout (default: 90s)

**Core Functionality**

- [ ]  ProcessorFactory implementation with `DEFAULT_PORT` input and `DEFAULT_PORT` + `REJECTED_PORT` outputs
- [ ]  Evaluate expressions in feature context for dynamic URL/headers/body
- [ ]  Execute HTTP request using `reqwest::blocking::Client`
- [ ]  Store response body, status code, headers as feature attributes
- [ ]  Route failed requests to `REJECTED_PORT` with error message
- [ ]  Category: "Web"

**Tests**

- [ ]  Parameter parsing and validation
- [ ]  Expression evaluation for URL construction
- [ ]  Request/response flow with mock server
- [ ]  Error handling and rejection routing
- [ ]  End-to-end workflow YAML test

---

**Phase 2: Authentication**

- [ ]  Basic authentication (username/password)
- [ ]  Bearer token authentication
- [ ]  API key authentication (header or query param)
- [ ]  HTTPS certificate verification toggle
- [ ]  Redirect following configuration
- [ ]  User-Agent customization

---

**Phase 3: Advanced Requests**

- [ ]  Additional HTTP methods (COPY, LOCK, MKCOL, MOVE, PROPFIND, PROPPATCH, UNLOCK)
- [ ]  Multipart form data uploads
- [ ]  File upload from storage paths
- [ ]  Mixed string/file parts with MIME types
- [ ]  Form URL-encoded body option
- [ ]  Binary request body support

---

**Phase 4: Response Handling**

- [ ]  Save response to file instead of attribute
- [ ]  Dynamic output file path from expressions
- [ ]  Binary response encoding
- [ ]  Response body size limits
- [ ]  Automatic encoding detection from headers
- [ ]  Large response streaming

---

**Phase 5: Reliability**

- [ ]  Retry on failure with configurable max attempts
- [ ]  Exponential backoff with initial delay
- [ ]  Retry on specific error types (5xx, network errors)
- [ ]  Honor Retry-After header
- [ ]  Rate limiting (requests per interval)
- [ ]  Request timing options (burst vs distributed)
- [ ]  Connection pooling and reuse

---

**Phase 6: Observability**

- [ ]  Request duration attribute
- [ ]  Final URL after redirects
- [ ]  Retry count tracking
- [ ]  Protocol version used
- [ ]  Bytes transferred
- [ ]  Integration with event hub for error logging

---

**Phase 7: Future Features (For future and not to be done this time)**

- [ ]  Proxy support (HTTP/HTTPS)
- [ ]  Proxy authentication
- [ ]  Client certificates (PKCS#12, PEM)
- [ ]  Private key password handling
- [ ]  Cookie jar session persistence
- [ ]  Request signing (AWS Signature V4)

## **Prerequisites (Please understand them before using reqwest)**

- The reqwest dependency is already available in `runtime/action-processor/Cargo.toml`
- Pattern reference: `xml/schema_fetcher.rs` demonstrates reqwest usage
- Pattern reference: `feature/filter.rs` demonstrates Processor with expressions
- Pattern reference: `center_point_replacer.rs` demonstrates `REJECTED_PORT` usage

# Bare Minimum Test Criteria for Phase 1

- [ ]  Please make one PR minimum per Phase and ask for review.
- [ ]  Please make sure that the actions and their parameters has proper description attached.
- [ ]  Unit tests and integration tests are present in the PRs
- [ ]  Make sure there's a Workflow YAML example
- [ ]  Expression evaluation for all dynamic values
- [ ]  Working HTTPCaller action registered in engine
- [ ]  Proper error handling with REJECTED_PORT routing
- [ ]  i18n entries for all supported languages