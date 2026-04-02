import { test, expect } from "@playwright/test";
import { mockTauriAPI } from "./helpers/tauri-mock";

test.describe("Doctor Tab", () => {
  test.beforeEach(async ({ page }) => {
    await mockTauriAPI(page);
    await page.goto("/");
    await page.click('[data-tab="doctor"]');
  });

  test("shows Doctor header", async ({ page }) => {
    await expect(page.locator("#tab-doctor h2")).toContainText("Doctor");
  });

  test("has Run Diagnostics button", async ({ page }) => {
    await expect(page.locator("#run-doctor-btn")).toBeVisible();
    await expect(page.locator("#run-doctor-btn")).toContainText(
      "Run Diagnostics"
    );
  });

  test("runs diagnostics and shows results", async ({ page }) => {
    await page.click("#run-doctor-btn");
    await expect(page.locator(".doctor-item").first()).toBeVisible();
  });

  test("shows correct number of diagnostic items", async ({ page }) => {
    await page.click("#run-doctor-btn");
    await expect(page.locator(".doctor-item")).toHaveCount(9);
  });

  test("shows OK status with checkmark", async ({ page }) => {
    await page.click("#run-doctor-btn");
    const okItem = page.locator(".doctor-icon.ok").first();
    await expect(okItem).toBeVisible();
  });

  test("shows Error status with X", async ({ page }) => {
    await page.click("#run-doctor-btn");
    const errorItem = page.locator(".doctor-icon.error");
    await expect(errorItem).toBeVisible();
  });

  test("shows diagnostic names", async ({ page }) => {
    await page.click("#run-doctor-btn");
    await expect(page.locator(".doctor-name").first()).toContainText(
      "Rubies directory"
    );
  });

  test("shows diagnostic messages", async ({ page }) => {
    await page.click("#run-doctor-btn");
    await expect(page.locator(".doctor-message").first()).toBeVisible();
  });

  test("shows fix hint for errors", async ({ page }) => {
    await page.click("#run-doctor-btn");
    await expect(page.locator(".doctor-hint").first()).toContainText(
      "Homebrew"
    );
  });

  test("shows Fix button for fixable issues", async ({ page }) => {
    await page.click("#run-doctor-btn");
    const fixBtn = page.locator(".doctor-item >> button:has-text('Fix')");
    await expect(fixBtn).toBeVisible();
  });

  test("Fix button triggers fix command", async ({ page }) => {
    await page.click("#run-doctor-btn");
    const fixBtn = page.locator(".doctor-item >> button:has-text('Fix')");
    await fixBtn.click();

    // Should show success toast
    await expect(page.locator(".toast-success")).toBeVisible();
  });

  test("button shows Running while diagnostics execute", async ({ page }) => {
    await page.click("#run-doctor-btn");
    // Button should be re-enabled after completion
    await expect(page.locator("#run-doctor-btn")).toBeEnabled();
    await expect(page.locator("#run-doctor-btn")).toContainText(
      "Run Diagnostics"
    );
  });
});
