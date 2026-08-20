# Load testing

`k6/graphql.js` drives a realistic query/mutation mix (list projects, list
jobs, create + update a project) against a running server, ramping 0 -> 10
-> 0 virtual users over ~3 minutes.

It is not run in CI; run it locally or against a staging deployment.

## Run

```sh
# install k6: https://k6.io/docs/get-started/installation/
k6 run \
  -e BASE_URL=http://localhost:8080 \
  -e TOKEN=<bearer token> \
  -e WORKSPACE_ID=<workspace id> \
  loadtest/k6/graphql.js
```

To assert against the metrics added in `internal/app/otel`, run the server
with `REEARTH_FLOW_METRICS=prometheus` (scrapes on `:9464/metrics` by
default; override with `REEARTH_FLOW_METRICS_PROMETHEUSADDR`) and watch
`reearth_flow_api_graphql_request_duration_milliseconds`,
`reearth_flow_api_graphql_accounts_calls`, and
`reearth_flow_api_graphql_redis_commands` move as the load test runs.
