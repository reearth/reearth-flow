# Refactor Code

Refactor the selected code to improve quality while preserving functionality.

## Guidelines

1. **Maintain existing functionality**
   - All tests must continue to pass
   - Public APIs remain compatible (or deprecate gracefully)
   - Behavior is preserved

2. **Improve readability and maintainability**
   - Clear, descriptive names
   - Appropriate abstraction levels
   - Single Responsibility Principle
   - Reduced complexity and nesting

3. **Follow project patterns and conventions**
   - **Engine**: Rust idioms, trait patterns, error handling with `thiserror`
   - **Server**: Clean Architecture, DDD patterns, repository interfaces
   - **UI**: React patterns, hooks, component composition

4. **Reduce complexity where possible**
   - Simplify conditional logic
   - Extract complex expressions
   - Break up large functions/methods
   - Reduce coupling between components

5. **Preserve or improve test coverage**
   - Run existing tests to ensure they pass
   - Add tests for refactored code if needed
   - Update tests if interfaces changed

## Common Refactoring Patterns

**Extract Function/Method**:
- When logic is repeated
- When a code block has a clear purpose
- When complexity needs to be hidden

**Extract Component** (UI):
- When JSX becomes too nested
- When logic is reusable
- When responsibilities can be separated

**Introduce Parameter/Generic**:
- To reduce duplication
- To increase flexibility
- To support testing

**Replace Conditional with Polymorphism**:
- When switch/if-else chains are complex
- When behavior varies by type
- When new variants are expected

**Simplify Nested Conditionals**:
- Early returns/guards
- Extract to well-named functions
- Use pattern matching (Rust) or similar

## Things to Avoid

- Don't refactor and add features simultaneously
- Don't over-engineer for hypothetical future needs
- Don't add unnecessary abstractions
- Don't break backward compatibility without deprecation

## Process

1. **Explain the refactoring goal** - What are we improving and why?
2. **Show the refactored code** - Complete implementation
3. **Explain key changes** - What's different and why it's better
4. **Run quality checks**:
   - Engine: `cargo make format && cargo make clippy && cargo make test`
   - Server: `make lint && make test`
   - UI: `yarn lint && yarn type && yarn format:write && yarn test --run`
5. **Verify behavior** - Confirm functionality is preserved
