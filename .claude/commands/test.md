# Generate Tests

Analyze the recent code changes and generate appropriate tests.

## Steps

1. **Identify what needs testing**
   - New functions/methods
   - Changed business logic
   - Edge cases and error paths
   - Integration points

2. **Generate appropriate tests**
   - Follow the project's testing patterns
   - Use the correct testing framework for each component
   - Include setup/teardown as needed

3. **Test Types by Component**

   **Engine (Rust)**:
   - Unit tests with `#[cfg(test)]` and `#[test]`
   - Integration tests in `runtime/tests/` with YAML fixtures
   - Use `pretty_assertions` for better diffs
   - Test both success and error paths

   **Server (Go)**:
   - Unit tests with `*_test.go` files
   - Use `testify` for assertions and mocking
   - Integration tests in `e2e/` directory
   - Test repository implementations with mock data

   **UI (React/TypeScript)**:
   - Unit tests with Vitest
   - Component tests with Testing Library
   - Mock GraphQL with MSW (Mock Service Worker)
   - Test user interactions, not implementation details
   - Check accessibility with `@testing-library/jest-dom`

4. **Ensure coverage of**
   - Happy path scenarios
   - Edge cases and boundary conditions
   - Error scenarios and error handling
   - Integration points with other components

5. **Run tests**
   - Engine: `cargo make test`
   - Server: `make test`
   - UI: `yarn test --run`

Provide the test code with clear explanations of what each test validates.
