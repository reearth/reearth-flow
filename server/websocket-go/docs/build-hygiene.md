# Build hygiene — toolchain pins & govulncheck triage (WS6)

## 1. golangci-lint for a go1.25 target

`server/websocket-go` is a standalone `go 1.25` module, deliberately not in the
repo root `go.work` (which is `go 1.24.10` and lists only the other server
modules). It still targets go1.25, so the lint constraint below applies. A
golangci-lint binary *built with go1.24* refuses to analyze a go1.25 target —
golangci-lint can only lint Go versions **≤** the Go it was compiled with.

- The locally-installed `golangci-lint 2.3.0 (built with go1.24.5)` therefore
  **cannot** lint this module.
- **Resolution: require golangci-lint ≥ v2.4.0**, which is the first release built
  with go1.25 and the first to support a go1.25 target. The official v2.4.0+
  binaries are built with go1.25.

  Sources:
  - <https://github.com/golangci/golangci-lint/issues/5873> ("since v2.4.0
    golangci-lint supports go1.25 if compiled with go1.25; official binaries are
    built with go1.25")
  - <https://golangci-lint.run/docs/product/changelog/>

This is pinned in CI (`deploy/ci_websocket.go.yml.draft`) via the
`golangci/golangci-lint-action` `version:` input — **not** installed
system-wide. Do not downgrade the module's go directive to satisfy an older
linter; bump the linter instead.

## 2. govulncheck triage

`go run golang.org/x/vuln/cmd/govulncheck@latest ./...` (run 2026-06, toolchain
go1.25.4) reports three buckets:

| Bucket | Count | Where | Triage |
|---|---:|---|---|
| **Symbol Results** (code calls the vulnerable symbol) | 15 | **Go standard library only** (`html/template`, `crypto/tls`, `crypto/x509`, `net`, `net/url`, `net/http`) | **Fixed by bumping the Go toolchain** to ≥ go1.25.10. None are in any third-party dependency. |
| **Package Results** (imported, not called) | 6 | **Go standard library only** (`net/mail`, `net`, `net/http/httputil`, `os`, `internal/syscall/unix`) | Same — Go toolchain bump. Not reachable from our code. |
| **Module Results** (required, not called) | 15 | **`golang.org/x/crypto@v0.51.0`** (every one) | Not called by our code; resolved by `go get golang.org/x/crypto@v0.52.0` at the next tidy. Low priority — no reachable path. |

### Key conclusions

- **Every "your code is affected" vulnerability is a standard-library issue**,
  fixed purely by building with a patched Go toolchain (≥ **go1.25.10**, the
  highest fix version any finding lists). The shipping binary's exposure is
  entirely the builder's Go patch version.
  - **Action:** the Dockerfile builder is `golang:1.25` (a moving tag that
    resolves to the latest 1.25.x patch at build time), and CI uses
    `actions/setup-go` with `go-version: '1.25'` (latest patch). Pin to a
    specific `1.25.x ≥ 1.25.10` once that patch is released/available on the
    runners; until then the floating `1.25` tag picks up patches automatically.

- **No vulnerability traces to a third-party dependency that our code calls.**
  In particular, the test-only modules **`github.com/fsouza/fake-gcs-server` and
  `github.com/alicebob/miniredis/v2` appear in ZERO vuln buckets** — there is
  nothing to suppress for the test tree, and nothing leaks into the shipping
  binary from them (they are imported only by `_test.go` files).

- The single non-stdlib finding (`golang.org/x/crypto@v0.51.0`, 15 advisories,
  all **not called**) is a transitive dep of the GCS/oauth/grpc stack, **not** a
  test-only dep. It is safe to defer (no reachable path) but should be picked up
  by `go get golang.org/x/crypto@latest && go mod tidy` in routine maintenance.

- **No blanket suppression** (`//govulncheck:ignore`, `-show=...` muting) was
  added. The clean path is the toolchain bump + a routine `x/crypto` upgrade,
  both of which eliminate the findings rather than hiding them.

### Reproduce

```bash
cd server/websocket-go
go run golang.org/x/vuln/cmd/govulncheck@latest ./...            # summary
go run golang.org/x/vuln/cmd/govulncheck@latest -show verbose ./...  # per-bucket
```

Exit code 3 = vulnerabilities found. CI treats a NON-stdlib *called* vuln as a
hard failure; stdlib-only findings are a soft signal to bump the builder patch
(see CI draft comments).
