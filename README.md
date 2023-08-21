# Opxs - Omnius Anything Service (API)
[![test](https://github.com/omnius-labs/opxs-api/actions/workflows/test.yml/badge.svg)](https://github.com/omnius-labs/opxs-api/actions/workflows/test.yml)

## Development

### Local

https://localhost.omnius-labs.com/api/docs/

```sh
# start opxs-api
RUN_MODE=local cargo make watch
```

```sh
# start postgres
docker compose up --build
```

## References
- https://github.com/tamasfe/aide/tree/master/examples/example-axum
