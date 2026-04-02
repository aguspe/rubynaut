import { test, expect } from "@playwright/test";
import { mockTauriAPI } from "./helpers/tauri-mock";

test.describe("Dashboard", () => {
  test.beforeEach(async ({ page }) => {
    await mockTauriAPI(page);
    await page.goto("/");
  });

  test("shows stat cards with correct values", async ({ page }) => {
    await expect(page.locator("#stat-installed")).toContainText("2");
    await expect(page.locator("#stat-active")).toContainText("4.0.2");
    await expect(page.locator("#stat-available")).toContainText("2");
  });

  test("displays installed Ruby versions", async ({ page }) => {
    await expect(page.locator(".version-number").first()).toContainText(
      "4.0.2"
    );
    await expect(page.locator(".version-number").nth(1)).toContainText(
      "3.3.6"
    );
  });

  test("shows Active badge on active version", async ({ page }) => {
    const activeItem = page.locator(".version-item.active-version");
    await expect(activeItem).toBeVisible();
    await expect(activeItem.locator(".badge-active")).toContainText("Active");
  });

  test("shows Global badge on global version", async ({ page }) => {
    const firstVersion = page.locator(".version-block").first();
    await expect(firstVersion.locator(".badge-global")).toContainText("Global");
  });

  test("has Gems button on each version", async ({ page }) => {
    const gemsButtons = page.locator(
      ".version-actions .btn-ghost >> text=Gems"
    );
    await expect(gemsButtons).toHaveCount(2);
  });

  test("has Projects button on each version", async ({ page }) => {
    const projectsButtons = page.locator(
      ".version-actions .btn-ghost >> text=Projects"
    );
    await expect(projectsButtons).toHaveCount(2);
  });

  test("has Use dropdown on each version", async ({ page }) => {
    const useButtons = page.locator(".version-switch-btn");
    await expect(useButtons).toHaveCount(2);
  });

  test("has Remove button on each version", async ({ page }) => {
    const removeButtons = page.locator(
      ".version-actions .btn-danger >> text=Remove"
    );
    await expect(removeButtons).toHaveCount(2);
  });

  test("toggles Gems panel on click", async ({ page }) => {
    const gemsBtn = page
      .locator(".version-block")
      .first()
      .locator("text=Gems");
    await gemsBtn.click();

    // Panel should appear with gem items
    const panel = page.locator("#panel-4\\.0\\.2");
    await expect(panel).not.toBeEmpty();
    await expect(panel.locator(".gem-item").first()).toBeVisible();

    // Click again to close
    await gemsBtn.click();
    await expect(panel).toBeEmpty();
  });

  test("Gems panel shows gem names and versions", async ({ page }) => {
    const gemsBtn = page
      .locator(".version-block")
      .first()
      .locator("text=Gems");
    await gemsBtn.click();

    const panel = page.locator("#panel-4\\.0\\.2");
    await expect(panel.locator(".gem-name").first()).toBeVisible();
    await expect(panel.locator(".gem-version").first()).toBeVisible();
  });

  test("Gems panel has install input", async ({ page }) => {
    const gemsBtn = page
      .locator(".version-block")
      .first()
      .locator("text=Gems");
    await gemsBtn.click();

    await expect(page.locator("#gem-install-input")).toBeVisible();
    await expect(page.locator("#gem-version-input")).toBeVisible();
  });

  test("Gems panel has filter input", async ({ page }) => {
    const gemsBtn = page
      .locator(".version-block")
      .first()
      .locator("text=Gems");
    await gemsBtn.click();

    await expect(page.locator("#gems-search")).toBeVisible();
  });

  test("toggles Projects panel on click", async ({ page }) => {
    const projectsBtn = page
      .locator(".version-block")
      .first()
      .locator("text=Projects");
    await projectsBtn.click();

    const panel = page.locator("#panel-4\\.0\\.2");
    await expect(panel).not.toBeEmpty();

    // Click again to close
    await projectsBtn.click();
    await expect(panel).toBeEmpty();
  });

  test("switching from Gems to Projects replaces panel", async ({ page }) => {
    const block = page.locator(".version-block").first();
    await block.locator("text=Gems").click();
    await expect(
      page.locator("#panel-4\\.0\\.2 .gem-item").first()
    ).toBeVisible();

    await block.locator("text=Projects").click();
    // Gems should be gone, projects should be visible
    await expect(page.locator("#panel-4\\.0\\.2 .gem-item")).toHaveCount(0);
  });

  test("opens Use dropdown menu", async ({ page }) => {
    const useBtn = page.locator(".version-switch-btn").first();
    await useBtn.click();

    const menu = page.locator(".dropdown-menu").first();
    await expect(menu).toBeVisible();
    await expect(
      menu.locator("text=Set as Global Default")
    ).toBeVisible();
    await expect(
      menu.locator("text=Set as Local (Project)")
    ).toBeVisible();
  });

  test("dropdown closes when clicking outside", async ({ page }) => {
    await page.locator(".version-switch-btn").first().click();
    await expect(page.locator(".dropdown-menu").first()).toBeVisible();

    await page.locator("#tab-dashboard h2").click();
    await expect(page.locator(".dropdown-menu").first()).toHaveClass(/hidden/);
  });
});
