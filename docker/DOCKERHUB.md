# rangular demo (browser SPA)

Static Leptos CSR / wasm demo for the **rangular** library: fixture components compiled AOT and served as a single-page app.

**Product:** `rangular` (library demo image)
Source: https://github.com/Interchouette-ITC/rangular

Docker Hub: [`interchouette/rangular-demo`](https://hub.docker.com/r/interchouette/rangular-demo)
Org GHCR: `ghcr.io/interchouette-itc/rangular-demo`

Runtime: nginx (unprivileged) serving Trunk `dist/`.

Live: https://rangular.interchouette.net/

## Tags

| Tag       | Meaning                                          |
| --------- | ------------------------------------------------ |
| `:dev`    | Tip of `dev`                                     |
| `:X.Y.Z`  | Stable GitHub Release matching workspace version |
| `:latest` | Same digest as the latest stable semver          |

```bash
docker pull interchouette/rangular-demo:dev
docker run --rm -p 8080:8080 interchouette/rangular-demo:dev
# → http://127.0.0.1:8080/
```

## Render

Deploy as a **Web Service** (Docker runtime) from this Dockerfile. Set custom domain `rangular.interchouette.net`. Render injects `PORT`; the entrypoint rewrites nginx to match.
