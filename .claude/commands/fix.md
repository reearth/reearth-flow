# Debug and Fix Issue

Help diagnose and fix the current issue systematically.

## Steps

1. **Analyze the problem**
   - Review error messages and stack traces
   - Examine relevant code and context
   - Check logs and debug output
   - Identify symptoms vs root cause

2. **Identify root cause**
   - Trace the error back to its origin
   - Check assumptions and invariants
   - Look for recent changes that might have introduced the issue
   - Consider environment or configuration issues

3. **Propose fix with explanation**
   - Provide complete code changes
   - Explain why this fixes the root cause
   - Describe the approach taken

4. **Consider side effects and edge cases**
   - Will this fix break anything else?
   - Are there similar issues elsewhere?
   - Does this handle all edge cases?
   - Performance implications

5. **Suggest tests to prevent regression**
   - Unit tests for the specific bug
   - Integration tests if cross-component
   - Edge cases that were missed

6. **Run quality checks**
   - Engine: `cargo make format && cargo make clippy && cargo make test`
   - Server: `make lint && make test`
   - UI: `yarn lint && yarn type && yarn format:write && yarn test --run`

## Component-Specific Debugging

**Engine (Rust)**:
- Check intermediate data in feature-store: `<working_dir>/projects/<project>/jobs/<job_id>/feature-store/<edge_id>.jsonl`
- Review action logs: `<working_dir>/projects/<project>/jobs/<job_id>/action-log/`
- Use `RUST_BACKTRACE=1` for detailed stack traces
- Check thread safety and ownership issues

**Server (Go)**:
- Review MongoDB queries and indexes
- Check GraphQL resolver logic
- Verify authentication/authorization
- Look for N+1 query patterns

**UI (React/TypeScript)**:
- Check browser console for errors
- Verify GraphQL types are in sync (`yarn gql`)
- Review state management (Jotai/TanStack Query/Yjs)
- Check for unnecessary re-renders
- Test WebSocket connection status

## Monorepo Debugging

- If the issue spans components, identify the integration point
- Check data flow: UI → Server → Engine
- Verify environment variables are set correctly
- Review service startup order (MongoDB → API → WebSocket → UI)
