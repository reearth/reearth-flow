# Build hygiene — toolchain pins & govulncheck triage (WS6)

## 1. golangci-lint for a go1.26 target

`server/websocket-go` is a standalone `go 1.26` module, deliberately not in the
repo root `go.work` (which is `go 1.24.10` and lists only the other server
modules). It targets go1.26, so the lint constraint below applies. A
golangci-lint binary *built with go1.25* refuses to analyze a go1.26 target —
golangci-lint can only lint Go versions **≤** the Go it was compiled with.

- The locally-installed `golangci-lint 2.3.0 (built with go1.24.5)` therefore
  **cannot** lint this module.
- **Resolution: require golangci-lint ≥ v2.13.0**, which is built with go1.26
  and supports a go1.26 target.

  Sources:
  - <https://github.com/golangci/golangci-lint/releases/tag/v2.13.0>
  - <https://golangci-lint.run/docs/product/changelog/>

This is pinned in CI (`.github/workflows/ci_websocket_go.yml`) via the
`golangci/golangci-lint-action` `version:` input — **not** installed
system-wide. Do not downgrade the module's go directive to satisfy an older
linter; bump the linter instead.

## 2. govulncheck triage

A historical `go run golang.org/x/vuln/cmd/govulncheck@latest ./...` scan (run
2026-06 with toolchain go1.25.4) reported three buckets:

| Bucket | Count | Where | Triage |
|---|---:|---|---|
| **Symbol Results** (code calls the vulnerable symbol) | 15 | **Go standard library only** (`html/template`, `crypto/tls`, `crypto/x509`, `net`, `net/url`, `net/http`) | **Fixed by bumping the Go toolchain** to ≥ go1.25.10. None are in any third-party dependency. |
| **Package Results** (imported, not called) | 6 | **Go standard library only** (`net/mail`, `net`, `net/http/httputil`, `os`, `internal/syscall/unix`) | Same — Go toolchain bump. Not reachable from our code. |
| **Module Results** (required, not called) | 15 | **`golang.org/x/crypto@v0.51.0`** (every one) | Not called by our code; resolved by `go get golang.org/x/crypto@v0.52.0` at the next tidy. Low priority — no reachable path. |

### Key conclusions

- CI later found five called vulnerabilities in `golang.org/x/net@v0.57.0`
  (GO-2026-6603, GO-2026-6610, GO-2026-6611, GO-2026-6612, and GO-2026-6617).
  They are fixed by `golang.org/x/net@v0.60.0`; that release requires Go 1.26.
- The module and CI therefore target Go 1.26, using the latest 1.26.x patch.
  CI also uses golangci-lint v2.13.0, which supports that target.
- The x/net upgrade also updates its required `golang.org/x/crypto`, `x/sync`,
  `x/sys`, and `x/text` versions. Keep the CI policy of hard-failing on called
  third-party vulnerabilities; do not suppress findings.

- **No blanket suppression** (`//govulncheck:ignore`, `-show=...` muting) was
  added. The clean path is to upgrade vulnerable dependencies and the toolchain,
  not hide findings.

### Reproduce

```bash
cd server/websocket-go
go run golang.org/x/vuln/cmd/govulncheck@latest ./...            # summary
go run golang.org/x/vuln/cmd/govulncheck@latest -show verbose ./...  # per-bucket
```

Exit code 3 = vulnerabilities found. CI treats a NON-stdlib *called* vuln as a
hard failure; stdlib-only findings are a soft signal to bump the builder patch
(see CI workflow comments).
