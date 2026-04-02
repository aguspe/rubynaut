import { test, expect } from "@playwright/test";
import { mockTauriAPI } from "./helpers/tauri-mock";

test.describe("Navigation", () => {
  test.beforeEach(async ({ page }) => {
    await mockTauriAPI(page);
    await page.goto("/");
  });

  test("loads the app with Dashboard as default tab", async ({ page }) => {
    await expect(page.locator("#tab-dashboard")).toBeVisible();
    await expect(page.locator("#tab-dashboard h2")).toContainText("Dashboard");
  });

  test("shows the Rubynaut logo in sidebar", async ({ page }) => {
    await expect(page.locator(".logo-icon")).toBeVisible();
    await expect(page.locator(".logo h1")).toContainText("Rubynaut");
  });

  test("has all navigation links", async ({ page }) => {
    await expect(page.locator('[data-tab="dashboard"]')).toBeVisible();
    await expect(page.locator('[data-tab="install"]')).toBeVisible();
    await expect(page.locator('[data-tab="projects"]')).toBeVisible();
    await expect(page.locator('[data-tab="doctor"]')).toBeVisible();
    await expect(page.locator('[data-tab="settings"]')).toBeVisible();
  });

  test("switches to Install tab", async ({ page }) => {
    await page.click('[data-tab="install"]');
    await expect(page.locator("#tab-install")).toBeVisible();
    await expect(page.locator("#tab-dashboard")).not.toBeVisible();
  });

  test("switches to Projects tab", async ({ page }) => {
    await page.click('[data-tab="projects"]');
    await expect(page.locator("#tab-projects")).toBeVisible();
    await expect(page.locator("#tab-projects h2")).toContainText("Projects");
  });

  test("switches to Doctor tab", async ({ page }) => {
    await page.click('[data-tab="doctor"]');
    await expect(page.locator("#tab-doctor")).toBeVisible();
    await expect(page.locator("#tab-doctor h2")).toContainText("Doctor");
  });

  test("switches to Settings tab", async ({ page }) => {
    await page.click('[data-tab="settings"]');
    await expect(page.locator("#tab-settings")).toBeVisible();
    await expect(page.locator("#tab-settings h2")).toContainText("Settings");
  });

  test("highlights active nav link", async ({ page }) => {
    await expect(page.locator('[data-tab="dashboard"]')).toHaveClass(
      /active/
    );
    await page.click('[data-tab="install"]');
    await expect(page.locator('[data-tab="install"]')).toHaveClass(/active/);
    await expect(page.locator('[data-tab="dashboard"]')).not.toHaveClass(
      /active/
    );
  });

  test("shows platform badge in sidebar", async ({ page }) => {
    await expect(page.locator("#platform-badge")).toContainText("macOS ARM64");
  });

  test("shows version label", async ({ page }) => {
    await expect(page.locator(".version-label")).toContainText("v0.1.0");
  });
});
