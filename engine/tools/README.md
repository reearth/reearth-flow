# Tools

## Cesium

1. Copy to cesium/3dtiles

Move the folder folder to be distributed under examples/3dtiles.
examples/3dtiles/tileset.json and should be

2. Start local web server:

    ```bash
    cd cesium
    python -m http.server
    ```

3. Open the browser and visit `http://localhost:8000/`

## mvt

1. Copy to mvt/dist

2. Start local web server:

    ```bash
    cd mvt
    python -m http.server
    ```

3. Open the browser and visit `http://localhost:8000/`

## czml

1. Copy to json/sample.json

2. Start local web server:

    ```bash
    cd czml
    python -m http.server
    ```

3. Open the browser and visit `http://localhost:8000/`

## diagnostics-repro

Runs a workflow through the worker locally and prints the diagnostics artifact — the exact
`failedNodes` / `aggregatedDiagnostics` payload the frontend receives. No GCP, MongoDB, pubsub
or frontend needed; `--pubsub-backend noop` covers all of it.

1. Build the worker, from `engine/`:

    ```bash
    cargo build -p reearth-flow-worker --bin reearth-flow-worker
    ```

2. Run any workflow in this directory, or one of your own:

    ```bash
    cd tools/diagnostics-repro
    ./run.sh expr-fatal.yml
    ```

It prints how many times the action actually failed, the diagnostics payload, and the output
data the workflow wrote. **Check the output data too** — a diagnostics payload can look correct
while the data is silently wrong, which is how a demoted failure once shipped a fabricated `0`.

The bundled workflows cover the shapes worth comparing against: a per-feature fatal
(`expr-fatal.yml`), the same failure demoted by an `errorPolicy` code override
(`expr-fatal-code-override.yml`), a build-time factory error (`expr-factory-error.yml`), a run
that fails on some features and succeeds on the rest (`expr-partial-failure.yml`), and a fatal
alongside unrelated warnings (`fatal-plus-warns.yml`).
