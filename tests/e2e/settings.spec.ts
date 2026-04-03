import { test, expect } from "@playwright/test";
import { mockTauriAPI } from "./helpers/tauri-mock";

test.describe("Settings Tab", () => {
  test.beforeEach(async ({ page }) => {
    await mockTauriAPI(page);
    await page.goto("/");
    // Wait for init() to complete (dashboard loads)
    await expect(page.locator("#stat-installed")).toContainText("2", { timeout: 5000 });
    await page.click('[data-tab="settings"]');
  });

  test("shows Settings header", async ({ page }) => {
    await expect(page.locator("#tab-settings h2")).toContainText("Settings");
  });

  test("shows Shell Integration section", async ({ page }) => {
    await expect(page.locator("#tab-settings h3").first()).toContainText("Shell Integration");
  });

  test("detects and shows shell hook status", async ({ page }) => {
    await expect(page.locator(".shell-hook-item")).toHaveCount(3);
  });

  test("shows installed badge for zsh", async ({ page }) => {
    const zshItem = page.locator(".shell-hook-item").first();
    await expect(zshItem.locator(".shell-hook-name")).toContainText("zsh");
    await expect(zshItem.locator(".badge-installed")).toContainText(
      "Installed"
    );
  });

  test("shows Install Hook button for uninstalled shells", async ({
    page,
  }) => {
    const bashItem = page.locator(".shell-hook-item").nth(1);
    await expect(bashItem.locator("text=Install Hook")).toBeVisible();
  });

  test("has shell selector for manual setup", async ({ page }) => {
    await expect(page.locator("#shell-select")).toBeVisible();
  });

  test("shell selector has all shell options", async ({ page }) => {
    const options = page.locator("#shell-select option");
    await expect(options).toHaveCount(4);
  });

  test("Show Hook button reveals hook code", async ({ page }) => {
    await page.click("text=Show Hook");
    await expect(page.locator("#shell-hook-preview")).toBeVisible();
    await expect(page.locator("#shell-hook-preview")).toContainText(
      "rubynaut_switch"
    );
  });

  test("shows About section", async ({ page }) => {
    await expect(page.locator(".about-info")).toBeVisible();
    await expect(page.locator(".about-info strong")).toContainText("Rubynaut");
    await expect(page.locator(".about-info")).toContainText("MIT");
  });

  test("shows GitHub link", async ({ page }) => {
    await expect(page.locator("text=GitHub").first()).toBeVisible();
  });

  test("shows Report Issue link", async ({ page }) => {
    await expect(page.locator("text=Report Issue")).toBeVisible();
  });

  test("shows Contribute link", async ({ page }) => {
    await expect(page.locator("text=Contribute")).toBeVisible();
  });

  test("Install Hook button triggers install", async ({ page }) => {
    const installBtn = page
      .locator(".shell-hook-item")
      .nth(1)
      .locator("text=Install Hook");
    await installBtn.click();
    await expect(page.locator(".toast-success")).toBeVisible();
  });
});
