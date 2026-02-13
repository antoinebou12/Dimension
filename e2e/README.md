# E2E tests (Playwright)

Browser end-to-end tests for WASM demos. Requires Node.js and Playwright browsers.

## What the tests do

- **Render** (`tests/render.spec.ts`): Opens the render WASM demo at `/wasm-demo/`, checks the canvas is visible with width/height, ensures the error div is not shown, and takes a screenshot for visual regression.
- **Kinematics** (`tests/kinematics.spec.ts`): Opens the kinematics-demo WASM at `/wasm-demo/`, checks the canvas (800×600) is visible and the error div is not shown, verifies the hint text (orbit / IK / zoom), and that the page title matches the demo.

## Commands (from repo root)

- **All projects** (render, mathlib, kinematics/demo): `just e2e`
- **Kinematics-demo only**: `just e2e-kinematics`
- **Debug with UI**: `just e2e-ui`

## Projects

| Project      | App              | URL (when running)   |
|-------------|------------------|-----------------------|
| `render`    | render WASM quad | http://localhost:3000/wasm-demo/ |
| `mathlib`   | mathlib demos    | http://localhost:3001/wasm-demo/ |
| `kinematics`| kinematics WASM  | http://localhost:3002/wasm-demo/ |

## Run a single project

```bash
cd e2e && npm install && npx playwright test --project=kinematics
```

Install browsers once: `npx playwright install` (or `npx playwright install chromium` for Chromium only). In headless/Docker environments, install [Playwright system dependencies](https://playwright.dev/docs/ci#docker) if the browser fails to launch (e.g. missing `libatk-1.0`, `libX11`, etc.).

## Manual WASM verification with Playwright MCP

The [Playwright MCP server](https://github.com/microsoft/playwright-mcp) lets an AI agent (e.g. in Cursor) drive a browser via tools like `browser_navigate`, `browser_snapshot`, and `browser_click`. Use it for manual or exploratory checks of the WASM demos alongside the automated e2e tests.

1. **Add Playwright MCP to your MCP client** (e.g. Cursor Settings → MCP → Add new MCP Server). Use command type with:
   ```bash
   npx -y @playwright/mcp@latest
   ```
2. **Serve a WASM demo** from the repo root, e.g. `just wasm-kinematics-demo` or `just wasm-render-serve`.
3. **Use the MCP browser tools** to open the demo URL (e.g. `http://localhost:3000/wasm-demo/` or `http://localhost:3002/wasm-demo/`) and take snapshots or interact. This does not replace the automated e2e tests; it complements them for manual verification.
