# Opxs - Omnius Anything Service (Apps)
[![test](https://github.com/omnius-labs/opxs-apps-rs/actions/workflows/test.yml/badge.svg?branch=main)](https://github.com/omnius-labs/opxs-apps-rs/actions/workflows/test.yml)

## Development

### Local

https://localhost.omnius-labs.com/api/docs/

```sh
# start opxs-api
RUN_MODE=local AWS_PROFILE=opxs-dev AWS_REGION=us-east-1 cargo make watch
```

```sh
# start postgres
docker compose up --build
```

## References
- https://github.com/tamasfe/aide/tree/master/examples/example-axum
