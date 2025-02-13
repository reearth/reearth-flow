# Re:Earth Flow API

## Development

### Install toolchains
- Golang (stable)


## Usage

### Start DB
```console
$ make run-db
```

### Run Server
```console
$ go run ./cmd/reearth-flow
```

## Authorization System

Re:Earth Flow uses Role-Based Access Control (RBAC) to manage permissions. The system is built using Cerbos for policy enforcement.

### Authorization Configuration
All authorization-related definitions are managed in `api/internal/rbac/definitions.go`. This file contains:

- Resource definitions (e.g., project, workflow)
- Action definitions (e.g., read, edit)
- Role definitions (owner, maintainer, writer, reader)
- Permission mappings between resources, actions, and roles

To add or modify permissions:
1. Open `api/internal/rbac/definitions.go`
2. Add/modify resources in the `ResourceXXX` constants
3. Add/modify actions in the `ActionXXX` constants
4. Add/modify roles in the `roleXXX` constants
5. Update the permission mappings in the `DefineResources` function

### Deployment
The permission definitions are automatically synchronized with the Cerbos server's storage bucket when changes are merged into the main branch via CI. This ensures that the latest permission settings are always available for authorization checks.

### Implementation in Use Cases
Permission checks should be implemented in use case interactors.
