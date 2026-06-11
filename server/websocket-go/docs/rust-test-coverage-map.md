# Rust → Go acceptance-test coverage map

This maps every Rust `server/websocket/tests/*.rs` conformance test to the Go
test(s) in `server/websocket-go` that now cover the same behavior, per the global
acceptance gate (plan WS6.2 / finding C §6). It is the artifact the cutover
gate checks: **every Rust acceptance test must have a green Go equivalent before
production traffic shifts.**

Go test paths are relative to `server/websocket-go/`. Run a named test with
`go test ./<pkg>/ -run <TestName>`.

| Rust test file | What it proves | Go test(s) | Status |
|---|---|---|---|
| `e2e_ws_test.rs` (`test_e2e_collaborative_editing_lifecycle`) | Full wire: SyncStep1→SyncStep2, multi-client Update relay, awareness | `internal/server/e2e_wire_test.go` → `TestE2EWireConformance`, `TestE2EWireDocIDNormalization`; also `internal/server/ws_test.go` → `TestSyncHandshake`, `TestDocIDNormalizationRoutesToSameRoom` | ✅ covered |
| `e2e_ws_test.rs` (`test_e2e_auth_rejection`) | WS token rejected when protected | `internal/auth/verify_test.go` → `TestEnabledRejectsAuthorizedFalse`, `TestEnabled401Denies`, `TestEnabledEmptyTokenDenies` (+ fail-closed variants) | ✅ covered (unit-level; see gap G3) |
| `e2e_ws_test.rs` (`test_e2e_api_secret_enforcement`) | `X-API-Secret` required on `/api/*` | `internal/http/middleware_test.go` → `TestAPISecretRejectsMissingHeader`, `TestAPISecretRejectsMismatch`, `TestAPISecretAcceptsExactMatch` | ✅ covered |
| `api_auth_test.rs` | `X-API-Secret` allow-all-when-unset / 401-when-set, constant-time | `internal/http/middleware_test.go` → `TestAPISecretAllowAllWhenUnset`, `TestAPISecretRejectsMissingHeader`, `TestAPISecretRejectsMismatch`, `TestAPISecretAcceptsExactMatch`, `TestAPISecretProdGuardFailsStartup`, `TestAPISecretDevAllowsEmpty` | ✅ covered (WS5) |
| `redis_channels_test.rs` | Pub/sub fan-out of write + awareness updates, self-filter, concurrency | `internal/redis/relay_test.go` → `TestTwoInstancesFanOutWithSelfFilter`, `TestXAddAwarenessType`, `TestCatchUpReplaysExistingStream`, `TestCatchUpDrainOrdering`, `TestLiveReaderXReadBlock`; `internal/redis/codec_test.go` → `TestXAddFieldOrderAndEncoding`, `TestParseEntry`, `TestPipelineXAddSharesOneTimestamp`; `internal/redis/writer_test.go` → `TestWriterDrainsInOrder`, `TestPublishDropsAndCountsWhenQueueFull` | ✅ covered (WS3) |
| `regression_spurious_versions_test.rs` | Rollback atomically prunes GCS updates `> target`; no resurrected future version (incl. crash mid-prune) | `internal/gcs/prune_recovery_test.go` → `TestCrashedPruneThenAppend_Phase1`, `TestCrashedPruneThenAppend_Phase2`; `internal/gcs/conformance_test.go` → `TestRunConformance`/`TestRunConformancePhase2` (ygo `RunConformance` PruneAfter target=0 crash subtest, Adapter implements CrashInjector+Reopener) | ✅ covered (WS4) |
| `regression_stale_canvas_test.rs` (bug: GCS 500/503/403 ≠ not-found) | A non-404 GCS error must PROPAGATE, not be swallowed as "doc not found" (the original stale-canvas root cause) | **none (code-correct, untested)** — `internal/gcs/kv.go` maps only `storage.ErrObjectNotExist`→`errNotFound` and propagates every other error, which is correct, but there is no Go test asserting a 500/503/403 propagates | ⚠️ **GAP G1** |
| `regression_stale_canvas_test.rs` (fix: 404 IS not-found; KV get returns data) | 404→None; success→bytes | `internal/gcs/conformance_test.go` (`TestRunConformance` exercises load-miss→empty + load-hit→bytes across Load/GetUpdate); `internal/gcs/adapter_test.go` → `TestLoadReadsV2Snapshot`, `TestBrotliRoundTrip` | ✅ covered (indirect) |
| `regression_stale_canvas_test.rs` (canvas-unfreeze signal) | `metadata.rollbackInProgress` toggles so the UI hides/re-shows the canvas | `internal/server/rollback_signal_test.go` → `TestSignalRollbackTogglesMetadata`, `TestRollbackClearSurvivesClientDisconnect`; `internal/http/router_test.go` → `TestRollbackSignalsLiveRoom`, `TestRollbackClearSurvivesClientDisconnect` | ✅ covered |
| `connection_reliability_test.rs` (lagged receiver recovers; counter never leaks on early return) | Slow/lagging subscriber recovers instead of dropping the stream; instance counters never leak on early return (the evict-race substrate) | `internal/redis/evict_test.go` → `TestSingleLockedEvictReconnectLosesNoUpdates`, `TestPublishRefusedDuringEvict`, `TestForceEvictDeletesStreamWithoutFlush`, `TestEnqueueNoOpForEvictingRoom`; `internal/redis/relay_test.go` → `TestCtxCancellationStopsReaders`, `TestRemoveHeartbeat`, `TestHeartbeatAndActiveCount` | ◑ partial — see gap G2 |
| `snapshot_management_test.rs` (no-change → 0 updates; change → exactly 1; cleanup keeps N; cleanup preserves state) | Snapshot/flush semantics + `cleanup_old_updates(keep)` | `internal/gcs/flusher_test.go` → `TestFlushRoomHoldsReadLock`, `TestFlushRoomNoopWhenEmpty`; `internal/gcs/retention_test.go` → `TestCompactKeepsMostRecentN`, `TestCompactNoopWhenFewUpdates`; `internal/http/router_test.go` → `TestCleanupUpdates`, `TestCreateSnapshotIsReadOnly`, `TestFlush` | ✅ covered |
| `regression_snapshot_retention_test.rs` (keeps exactly max; no-op under/at limit; hundreds; non-contiguous clocks) | Retention edge cases | `internal/gcs/retention_test.go` → `TestLogarithmicRetentionAlgorithm`, `TestDensityParameterImpact`, `TestRetentionEdgeCases`, `TestCompactKeepsMostRecentN`, `TestCompactNoopWhenFewUpdates`, `TestFirstZeroBit` | ✅ covered |
| `logarithmic_retention_test.rs` | Logarithmic retention curve / density | `internal/gcs/retention_test.go` → `TestLogarithmicRetentionAlgorithm`, `TestDensityParameterImpact`, `TestRetentionEdgeCases` | ✅ covered |
| `flush_to_gcs_test.rs` | Last-disconnect flush persists state to GCS | `internal/gcs/flusher_test.go` → `TestFlushRoomHoldsReadLock`, `TestFlushRoomNoopWhenEmpty`; `internal/redis/relay_test.go` → `TestLastInstanceEvictsFlushesThenDeletes`, `TestRoomActivatedWritesMarkerAndHeartbeat` | ✅ covered |
| `gcs_infra_test.rs` (`test_raw_kv_roundtrip`, `test_doc_v2_roundtrip`, `test_push_update_and_list`) | Golden object-name byte parity + KV/doc_v2 round-trip + push/list | `internal/gcs/layout_test.go` → `TestLegacyRootLayout_GoldenNames`, `TestLegacyRootLayout_UUIDDoubleHex`, `TestLegacyRootLayout_V1Keyspace`, `TestProjectFolderLayout_Names`, `TestValidateDocIDForPrefix`; `internal/gcs/adapter_test.go` → `TestBrotliRoundTrip`, `TestLoadReadsV2Snapshot`; `internal/gcs/conformance_test.go` → `TestRunConformance` (append→list) | ✅ covered (WS4) |
| `brotli.rs` | Brotli compress/decompress round-trip | `internal/gcs/adapter_test.go` → `TestBrotliRoundTrip` | ✅ covered |
| `error_handling_test.rs` | KV `is_not_found` discrimination (subset of stale-canvas) | (same as G1) | ⚠️ folded into **GAP G1** |
| `mock_server.rs`, `gcs_test_utils.rs`, `ydoc.rs` | Test harness/fixtures, not behavior | n/a (Go uses `fakestorage`, `miniredis`, ygo `crdt`/`sync` directly) | n/a |

## Genuinely uncovered behavior (gaps to close before/at cutover)

- **G1 — GCS non-404 error propagation is untested.** `internal/gcs/kv.go` is
  *correct* (only `storage.ErrObjectNotExist` becomes `errNotFound`; 500/503/403
  propagate), but no Go test asserts it the way Rust
  `regression_stale_canvas_test`/`error_handling_test` do. This is the original
  stale-canvas root cause (a transient 500 misread as "doc empty" wipes the
  canvas). **Recommended:** a `kv_test.go` that injects a non-404 from a faulting
  transport (or a closed/borked fake-gcs) and asserts `get` returns a non-nil,
  non-`errNotFound` error. Low effort; high value as a regression guard.

- **G2 — Lagged-subscriber RECOVERY (vs. drop) is not directly asserted.** The Go
  evict tests prove the *single-locked-evict* race and counter non-leak, but the
  Rust `connection_reliability_test::lagged_receiver_recovers_and_continues`
  specifically asserts that a *slow XREAD consumer* recovers and keeps streaming
  rather than exiting on lag. The Go reader uses `XREAD COUNT n BLOCK` with a
  per-reader advancing last-id (no bounded broadcast channel that can "lag" in the
  tokio-broadcast sense), so the failure mode is structurally different — but a
  test that backs up a reader and asserts no entries are skipped would close the
  parity explicitly. Medium effort.

- **G3 — WS auth is unit-tested, not e2e-tested over a live upgrade.** `test_e2e_auth_rejection` rejects at the WS *upgrade* against a live server +
  mock `/auth/verify`. The Go `internal/auth/verify_test.go` covers the
  `AuthFunc` decision matrix thoroughly (fail-closed on 401/500/timeout/empty),
  and `HandlerWithAPI` wiring is tested, but there is no end-to-end test that a
  protected-mode server *refuses the WebSocket upgrade* for a bad token. This is
  acceptable for Phase-1 cutover because **protected mode ships OFF** (finding G
  §4 decision) and is enabled as its own separately-gated launch — but the e2e
  upgrade-rejection test SHOULD exist before flipping `REEARTH_FLOW_WS_PROTECTED=true`. Tracked as a pre-requisite of runbook step 7, not of the
  initial traffic shift.

All other Rust acceptance tests have green Go equivalents.
