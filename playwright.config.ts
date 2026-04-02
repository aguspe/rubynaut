import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./tests/e2e",
  timeout: 30_000,
  retries: 0,
  use: {
    // Tauri uses WebKit on macOS/Linux, so we test with WebKit
    browserName: "webkit",
    // Connect to the Tauri dev server
    baseURL: "http://localhost:1420",
    screenshot: "only-on-failure",
    trace: "on-first-retry",
  },
  // Start the frontend file server before tests
  webServer: {
    command: "npx serve frontend/dist -l 1420 --no-clipboard",
    port: 1420,
    reuseExistingServer: true,
    timeout: 10_000,
  },
  reporter: [["html", { open: "never" }], ["list"]],
});
