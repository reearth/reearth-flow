# Review Code Changes

Review the current changes and provide:

1. **Code quality assessment**
   - Adherence to project patterns and conventions
   - Readability and maintainability
   - Appropriate abstraction levels

2. **Potential bugs or issues**
   - Logic errors
   - Edge cases not handled
   - Race conditions or concurrency issues

3. **Security concerns**
   - Input validation
   - Authentication/authorization
   - Injection vulnerabilities (SQL, XSS, etc.)
   - Secrets or credentials exposure

4. **Performance considerations**
   - Algorithmic efficiency
   - N+1 queries (Server GraphQL)
   - Unnecessary re-renders (UI React)
   - Memory usage (Engine Rust)

5. **Suggestions for improvement**
   - Better patterns or approaches
   - Code simplification opportunities
   - Documentation needs

6. **Tests that should be added**
   - Unit tests for new functionality
   - Integration tests for cross-component changes
   - Edge cases and error scenarios

## Component-Specific Focus

- **Engine (Rust)**: Memory safety, error handling, thread safety, action patterns
- **Server (Go)**: Clean architecture adherence, GraphQL resolver patterns, repository interfaces
- **UI (React/TypeScript)**: Component patterns, state management, accessibility, performance

## Monorepo Concerns

- Does this change affect other components?
- Are GraphQL schema changes synchronized?
- Is cross-component data flow preserved?
