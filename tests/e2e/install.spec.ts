import { test, expect } from "@playwright/test";
import { mockTauriAPI } from "./helpers/tauri-mock";

test.describe("Install Tab", () => {
  test.beforeEach(async ({ page }) => {
    await mockTauriAPI(page);
    await page.goto("/");
    await page.click('[data-tab="install"]');
  });

  test("shows Install Ruby header", async ({ page }) => {
    await expect(page.locator("#tab-install h2")).toContainText("Install Ruby");
    await expect(page.locator("#tab-install .tab-subtitle")).toContainText(
      "pre-built"
    );
  });

  test("displays available Ruby versions as cards", async ({ page }) => {
    await expect(page.locator(".version-card").first()).toBeVisible();
  });

  test("shows version numbers on cards", async ({ page }) => {
    const cards = page.locator(".version-card");
    await expect(cards).toHaveCount(4);
  });

  test("marks installed versions with badge", async ({ page }) => {
    const installedBadges = page.locator(".badge-installed");
    await expect(installedBadges.first()).toContainText("Installed");
  });

  test("marks uninstalled versions with Pre-built badge", async ({ page }) => {
    const prebuiltBadges = page.locator(".badge-prebuilt");
    await expect(prebuiltBadges.first()).toContainText("Pre-built");
  });

  test("shows Install button only on uninstalled versions", async ({
    page,
  }) => {
    // Cards with clickable install buttons (not the disabled "Installed" ones)
    const installButtons = page.locator(
      ".version-card >> button.btn-primary:has-text('Install')"
    );
    await expect(installButtons).toHaveCount(2); // 3.2.4 and 3.1.6
  });

  test("shows Installed button (disabled) on installed versions", async ({
    page,
  }) => {
    const installedButtons = page.locator(
      ".version-card >> button:has-text('Installed')"
    );
    await expect(installedButtons.first()).toBeDisabled();
  });

  test("progress container is hidden by default", async ({ page }) => {
    await expect(page.locator("#install-progress-container")).toHaveClass(
      /hidden/
    );
  });
});
