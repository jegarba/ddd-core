# ddd-core — the mold

Generic DDD + Hexagonal kernel for Rust backends. Zero business concepts —
see `infra-platform/docs/standards/ddd-kernel-mold.md` for the full design
doc. Not a deployed service, not an "Egyptian" — a library, consumed as a
`git` dependency by services that need it (today: `egyptians-ms-logto`,
`egyptians-ms-product`).

## What's in here

- `domain`: `Entity`/`AggregateRoot`, `ValueObject`, `DomainError`, `DomainEvent`/`EventBus`
- `application`: `ReadRepository`/`WriteRepository` ports (ISP), `QueryUseCase`/`CreateUseCase`,
  optional `Authenticator`/`Principal` auth port
- `infrastructure`: `InMemoryRepository` (tests), `RestResource` (generic Axum CRUD factory),
  RFC 7807 error mapping, standard middleware stack, observability (tracing/health/shutdown),
  optional caching adapter

## Using it

```toml
[dependencies]
ddd-core = { git = "https://github.com/jegarba/ddd-core.git" }
```

See `ddd-kernel-mold.md` §5 for the per-new-domain implementation checklist.
