# Rubynaut Web Architecture: Running as a Web Service

This document outlines how Rubynaut could be adapted to run as a web-based tool for environments where desktop app installation is not possible (Chromebooks, school computers, GitHub Codespaces).

## Current Architecture

```
+------------------+     Tauri IPC      +------------------+
|   Frontend       | <================> |   Rust Backend   |
|   (HTML/JS/CSS)  |   invoke/listen    |   (rubynaut-core)|
+------------------+                    +------------------+
        ^                                        |
        |                                        v
    Tauri Webview                          File System
                                          (~/.rubies/)
```

Key observation: the only coupling between the frontend and backend is two functions at the top of `app.js`:

```javascript
function invoke(...args) {
  return window.__TAURI__.core.invoke(...args);
}
function listen(...args) {
  return window.__TAURI__.event.listen(...args);
}
```

## Proposed Web Architecture

```
+------------------+     HTTP/SSE       +------------------+
|   Frontend       | <================> |   rubynaut-server |
|   (same HTML/JS) |   fetch/EventSource|   (Axum/Actix)    |
+------------------+                    +------------------+
        ^                                        |
        |                                        v
    Any Browser                           rubynaut-core
                                          File System
```

### Step 1: Transport Abstraction (Frontend)

Replace the `invoke()` and `listen()` functions with a transport abstraction:

```javascript
function invoke(command, args = {}) {
  if (window.__TAURI__) {
    return window.__TAURI__.core.invoke(command, args);
  }
  return fetch(`/api/${command}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(args),
  }).then(r => {
    if (!r.ok) return r.text().then(t => Promise.reject(t));
    return r.json();
  });
}

function listen(event, handler) {
  if (window.__TAURI__) {
    return window.__TAURI__.event.listen(event, handler);
  }
  const source = new EventSource(`/api/events/${event}`);
  source.onmessage = (e) => handler({ payload: JSON.parse(e.data) });
  return () => source.close();
}
```

This is a ~20-line change. The rest of `app.js` works unchanged.

### Step 2: HTTP API Server (New Crate)

Create `rubynaut-server` crate using Axum:

```
rubynaut-server/
  Cargo.toml
  src/
    main.rs     # Server entrypoint, route registration
    routes.rs   # Route handlers mapping to rubynaut-core
```

Routes map 1:1 to existing Tauri commands:

| Method | Route | Maps to |
|--------|-------|---------|
| GET | /api/get_installed_rubies | `rubynaut_core::get_installed_rubies()` |
| GET | /api/get_available_rubies | `rubynaut_core::get_available_rubies()` |
| GET | /api/get_active_version | `rubynaut_core::get_active_version()` |
| POST | /api/set_global_version | `rubynaut_core::set_global_version(version)` |
| POST | /api/install_ruby | `rubynaut_core::install_ruby(version)` |
| POST | /api/uninstall_ruby | `rubynaut_core::uninstall_ruby(version)` |
| GET | /api/run_doctor | `rubynaut_core::run_doctor()` |
| GET | /api/check_shell_hook | `rubynaut_core::check_shell_hook()` |
| ... | ... | ... |

Progress reporting uses Server-Sent Events (SSE):

| Route | Purpose |
|-------|---------|
| GET /api/events/install-progress | SSE stream for install progress |
| GET /api/events/bundle-progress | SSE stream for bundle progress |

### Step 3: Docker Image

```dockerfile
FROM rust:1.77 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release -p rubynaut-server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates tar && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/rubynaut-server /usr/local/bin/
COPY frontend/dist/ /app/frontend/dist/
EXPOSE 3000
ENV RUBYNAUT_BIND=0.0.0.0:3000
CMD ["rubynaut-server"]
```

### Step 4: GitHub Codespaces

`.devcontainer/devcontainer.json`:

```json
{
  "name": "Rubynaut",
  "image": "ghcr.io/aguspe/rubynaut-server:latest",
  "forwardPorts": [3000],
  "postCreateCommand": "rubynaut init --no-rails",
  "customizations": {
    "vscode": {
      "settings": {
        "terminal.integrated.defaultProfile.linux": "bash"
      }
    }
  }
}
```

## Security Considerations

### Bind to localhost by default

The server MUST bind to `127.0.0.1` by default. Exposing to `0.0.0.0` should require an explicit flag or environment variable. A version manager running on a public interface allows anyone to install, uninstall, or modify Ruby installations.

### Authentication

For shared environments (classrooms, CI servers), add optional token-based auth:

```
RUBYNAUT_AUTH_TOKEN=mysecrettoken rubynaut-server
```

All API requests must include `Authorization: Bearer mysecrettoken`.

### Read-only mode

For demonstration or documentation purposes:

```
RUBYNAUT_READ_ONLY=1 rubynaut-server
```

This disables install, uninstall, gem management, and config writes. Only read operations are permitted.

### No shell hook installation

In a web context, there is no persistent shell to hook into. The shell integration commands should return informational messages rather than attempting to modify files.

### File permission isolation

In Docker, the server runs as a non-root user. Ruby installations go to `/home/rubynaut/.rubies/`. The container filesystem is ephemeral unless a volume is mounted.

## What Would NOT Work in Web Mode

1. **Shell hook installation** — no persistent shell in a web context
2. **System PATH modification** — the browser cannot modify the user's environment
3. **Platform detection** — returns the server's platform, not the user's
4. **File picker dialog** — `pick_folder` uses a native dialog; web would need a path input field
5. **External link opening** — `openExternal` would need to open in a new tab instead

## Estimated Effort

| Component | Effort |
|-----------|--------|
| Frontend transport abstraction | 1 day |
| rubynaut-server crate (Axum) | 3-5 days |
| Docker image + CI | 1 day |
| Codespaces devcontainer | 0.5 day |
| Testing | 2 days |
| **Total** | **~8-10 days** |

## Recommendation

Start with the frontend transport abstraction (Step 1) as it's a minimal, zero-risk change that enables future web support. The `invoke()` abstraction can be merged independently, and all existing tests will pass since they mock `window.__TAURI__`.

The server crate (Step 2) should only be built when there's clear demand from bootcamp partners or educational institutions — it adds a maintenance burden (another crate, Docker builds, security surface) that is only justified by concrete user need.
