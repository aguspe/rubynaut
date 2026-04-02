import { test, expect } from "@playwright/test";
import { mockTauriAPI } from "./helpers/tauri-mock";

test.describe("Projects Tab", () => {
  test.beforeEach(async ({ page }) => {
    await mockTauriAPI(page);
    await page.goto("/");
    await page.click('[data-tab="projects"]');
  });

  test("shows Projects header", async ({ page }) => {
    await expect(page.locator("#tab-projects h2")).toContainText("Projects");
  });

  test("shows Open Project Folder button", async ({ page }) => {
    await expect(
      page.locator("text=Open Project Folder")
    ).toBeVisible();
  });

  test("shows detection source hints", async ({ page }) => {
    await expect(page.locator(".project-open-hint")).toBeVisible();
    await expect(page.locator(".project-open-hint")).toContainText(".ruby-version");
    await expect(page.locator(".project-open-hint")).toContainText("Gemfile");
  });

  test("displays tracked projects list", async ({ page }) => {
    await expect(page.locator("#tracked-projects-section")).toBeVisible();
    await expect(
      page.locator(".tracked-project-item")
    ).toHaveCount(2);
  });

  test("shows project names", async ({ page }) => {
    await expect(
      page.locator(".tracked-project-name").first()
    ).toContainText("myapp");
    await expect(
      page.locator(".tracked-project-name").nth(1)
    ).toContainText("legacy");
  });

  test("shows project paths", async ({ page }) => {
    await expect(
      page.locator(".tracked-project-path").first()
    ).toBeVisible();
  });

  test("shows version info per project", async ({ page }) => {
    await expect(
      page.locator(".tracked-project-version-number").first()
    ).toContainText("4.0.2");
  });

  test("shows Ready badge for installed version", async ({ page }) => {
    await expect(page.locator(".badge-ready").first()).toContainText("Ready");
  });

  test("shows Not Installed badge for missing version", async ({ page }) => {
    await expect(page.locator(".badge-missing-ruby")).toContainText(
      "Not Installed"
    );
  });

  test("shows Bundle button for projects with Gemfile", async ({ page }) => {
    // myapp has gemfile + version installed
    const bundleBtn = page
      .locator(".tracked-project-block")
      .first()
      .locator("text=Bundle");
    await expect(bundleBtn).toBeVisible();
  });

  test("shows Gems button for projects with Gemfile.lock", async ({
    page,
  }) => {
    // myapp has gemfile.lock
    const gemsBtn = page
      .locator(".tracked-project-block")
      .first()
      .locator("text=Gems");
    await expect(gemsBtn).toBeVisible();
  });

  test("shows Remove button on all projects", async ({ page }) => {
    const removeButtons = page.locator(
      ".tracked-project-actions >> text=Remove"
    );
    await expect(removeButtons).toHaveCount(2);
  });

  test("toggles project gems on Gems click", async ({ page }) => {
    const gemsBtn = page
      .locator(".tracked-project-block")
      .first()
      .locator("button:has-text('Gems')");
    await gemsBtn.click();

    const panel = page.locator("#project-panel-0");
    await expect(panel.locator(".gem-item").first()).toBeVisible();

    // Click again to close
    await gemsBtn.click();
    await expect(panel).toBeEmpty();
  });

  test("Bundle install shows log panel", async ({ page }) => {
    const bundleBtn = page
      .locator(".tracked-project-block")
      .first()
      .locator("text=Bundle");
    await bundleBtn.click();

    await expect(
      page.locator("#project-panel-0 .bundle-output")
    ).toBeVisible();
  });

  test("shows Install button for project with missing Ruby", async ({
    page,
  }) => {
    // legacy project has version not installed
    const installBtn = page
      .locator(".tracked-project-block")
      .nth(1)
      .locator("button.btn-primary:has-text('Install')");
    await expect(installBtn).toBeVisible();
  });
});
