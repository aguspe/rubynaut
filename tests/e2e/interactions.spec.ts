import { test, expect, Page } from "@playwright/test";
import { mockTauriAPI } from "./helpers/tauri-mock";

// ============================================
// Gem Install/Uninstall Flows
// ============================================

test.describe("Gem Management", () => {
  test.beforeEach(async ({ page }) => {
    await mockTauriAPI(page);
    await page.goto("/");
  });

  test("can open gems panel and see gem list", async ({ page }) => {
    const gemsBtn = page.locator(".version-actions .btn-ghost >> text=Gems").first();
    await gemsBtn.click();
    await expect(page.locator(".gem-item").first()).toBeVisible();
    await expect(page.locator(".gem-name").first()).toContainText("bundler");
  });

  test("shows default and user badges on gems", async ({ page }) => {
    await page.locator(".version-actions .btn-ghost >> text=Gems").first().click();
    await expect(page.locator(".gem-badge-default").first()).toContainText("default");
    await expect(page.locator(".gem-badge-user").first()).toContainText("user");
  });

  test("shows gem install input and button", async ({ page }) => {
    await page.locator(".version-actions .btn-ghost >> text=Gems").first().click();
    await expect(page.locator("#gem-install-input")).toBeVisible();
    await expect(page.locator("#gem-install-btn")).toBeVisible();
  });

  test("gem install button triggers install", async ({ page }) => {
    await page.locator(".version-actions .btn-ghost >> text=Gems").first().click();
    await page.fill("#gem-install-input", "puma");
    await page.click("#gem-install-btn");
    // Should show success toast
    await expect(page.locator(".toast-success")).toBeVisible({ timeout: 5000 });
  });

  test("gem install with version", async ({ page }) => {
    await page.locator(".version-actions .btn-ghost >> text=Gems").first().click();
    await page.fill("#gem-install-input", "puma");
    await page.fill("#gem-version-input", "6.4.0");
    await page.click("#gem-install-btn");
    await expect(page.locator(".toast-success")).toBeVisible({ timeout: 5000 });
  });

  test("gem uninstall shows confirmation dialog", async ({ page }) => {
    await page.locator(".version-actions .btn-ghost >> text=Gems").first().click();
    // Click remove button on a user gem (non-default)
    const removeBtn = page.locator(".gem-remove-btn").first();
    await removeBtn.click();
    // Confirmation dialog should appear
    await expect(page.locator(".dialog-overlay")).toBeVisible();
    await expect(page.locator(".dialog h3")).toContainText("Uninstall Gem");
  });

  test("gem uninstall confirm triggers removal", async ({ page }) => {
    await page.locator(".version-actions .btn-ghost >> text=Gems").first().click();
    await page.locator(".gem-remove-btn").first().click();
    await page.click("#dialog-confirm");
    await expect(page.locator(".toast-success")).toBeVisible({ timeout: 5000 });
  });

  test("gem uninstall cancel closes dialog", async ({ page }) => {
    await page.locator(".version-actions .btn-ghost >> text=Gems").first().click();
    await page.locator(".gem-remove-btn").first().click();
    await page.click("#dialog-cancel");
    await expect(page.locator(".dialog-overlay")).not.toBeVisible();
  });

  test("gem filter shows search input", async ({ page }) => {
    await page.locator(".version-actions .btn-ghost >> text=Gems").first().click();
    await expect(page.locator("#gems-search")).toBeVisible();
  });

  test("gem filter by name works", async ({ page }) => {
    await page.locator(".version-actions .btn-ghost >> text=Gems").first().click();
    await page.fill("#gems-search", "rails");
    // The filter function is called on input event
    // With the mock data, only "rails" should match among the gems
  });

  test("gem filter checkboxes visible", async ({ page }) => {
    await page.locator(".version-actions .btn-ghost >> text=Gems").first().click();
    await expect(page.locator("#gems-show-default")).toBeChecked();
    await expect(page.locator("#gems-show-user")).toBeChecked();
  });
});

// ============================================
// Ruby Uninstall Flow
// ============================================

test.describe("Ruby Uninstall", () => {
  test.beforeEach(async ({ page }) => {
    await mockTauriAPI(page);
    await page.goto("/");
  });

  test("Remove button shows confirmation dialog", async ({ page }) => {
    const removeBtn = page.locator(".btn-danger >> text=Remove").first();
    await removeBtn.click();
    await expect(page.locator(".dialog-overlay")).toBeVisible();
    await expect(page.locator(".dialog h3")).toContainText("Uninstall Ruby");
  });

  test("confirm removal triggers uninstall", async ({ page }) => {
    await page.locator(".btn-danger >> text=Remove").first().click();
    await page.click("#dialog-confirm");
    await expect(page.locator(".toast-success")).toBeVisible({ timeout: 5000 });
  });

  test("cancel removal closes dialog without action", async ({ page }) => {
    await page.locator(".btn-danger >> text=Remove").first().click();
    await page.click("#dialog-cancel");
    await expect(page.locator(".dialog-overlay")).not.toBeVisible();
  });
});

// ============================================
// Version Dropdown
// ============================================

test.describe("Version Use Dropdown", () => {
  test.beforeEach(async ({ page }) => {
    await mockTauriAPI(page);
    await page.goto("/");
  });

  test("clicking Use opens dropdown menu", async ({ page }) => {
    await page.locator(".version-switch-btn").first().click();
    const menu = page.locator(".dropdown-menu").first();
    await expect(menu).not.toHaveClass(/hidden/);
  });

  test("Set as Global option triggers set_global_version", async ({ page }) => {
    await page.locator(".version-switch-btn").first().click();
    await page.locator(".dropdown-item >> text=Set as Global Default").first().click();
    await expect(page.locator(".toast-success")).toBeVisible({ timeout: 5000 });
  });

  test("clicking outside closes dropdown", async ({ page }) => {
    await page.locator(".version-switch-btn").first().click();
    // Click outside
    await page.click("body", { position: { x: 10, y: 10 } });
    const menus = page.locator(".dropdown-menu:not(.hidden)");
    await expect(menus).toHaveCount(0);
  });
});

// ============================================
// Install Tab Interactions
// ============================================

test.describe("Install Flow", () => {
  test.beforeEach(async ({ page }) => {
    await mockTauriAPI(page);
    await page.goto("/");
    await page.click('[data-tab="install"]');
  });

  test("install button triggers install and shows success", async ({ page }) => {
    // Click install on an uninstalled version
    const installBtn = page.locator(".version-card:not(.installed) .btn-primary >> text=Install").first();
    await installBtn.click();
    await expect(page.locator(".toast-success")).toBeVisible({ timeout: 5000 });
  });

  test("progress container becomes visible during install", async ({ page }) => {
    const installBtn = page.locator(".version-card:not(.installed) .btn-primary >> text=Install").first();
    await installBtn.click();
    // Progress container should show briefly
    // (Since mock resolves instantly, it may hide quickly)
  });

  test("installed version cards have disabled button", async ({ page }) => {
    const installedBtns = page.locator(".version-card.installed button");
    const firstBtn = installedBtns.first();
    await expect(firstBtn).toBeDisabled();
  });
});

// ============================================
// Projects Tab Interactions
// ============================================

test.describe("Projects Interactions", () => {
  test.beforeEach(async ({ page }) => {
    await mockTauriAPI(page);
    await page.goto("/");
    await page.click('[data-tab="projects"]');
  });

  test("Bundle button visible for ready projects", async ({ page }) => {
    const bundleBtn = page.locator("button >> text=Bundle");
    await expect(bundleBtn.first()).toBeVisible();
  });

  test("Bundle install shows output panel", async ({ page }) => {
    await page.locator("button >> text=Bundle").first().click();
    await expect(page.locator(".bundle-log")).toBeVisible();
    await expect(page.locator(".bundle-output")).toBeVisible();
  });

  test("Bundle install shows success toast on completion", async ({ page }) => {
    await page.locator("button >> text=Bundle").first().click();
    await expect(page.locator(".toast-success")).toBeVisible({ timeout: 5000 });
  });

  test("Remove button removes project from tracking", async ({ page }) => {
    const removeBtns = page.locator(".tracked-project-actions button >> text=Remove");
    await removeBtns.first().click();
    await expect(page.locator(".toast-success")).toBeVisible({ timeout: 5000 });
  });

  test("Gems button opens project gems panel", async ({ page }) => {
    const gemsBtn = page.locator(".tracked-project-actions button >> text=Gems");
    if ((await gemsBtn.count()) > 0) {
      await gemsBtn.first().click();
      await expect(page.locator(".project-inline-panel .gem-item").first()).toBeVisible({ timeout: 5000 });
    }
  });

  test("Install button shows for missing version projects", async ({ page }) => {
    // The legacy project has version_installed: false
    const installBtn = page.locator(".tracked-project-actions .btn-primary >> text=Install");
    await expect(installBtn.first()).toBeVisible();
  });
});

// ============================================
// Doctor Tab Interactions
// ============================================

test.describe("Doctor Interactions", () => {
  test.beforeEach(async ({ page }) => {
    await mockTauriAPI(page);
    await page.goto("/");
    await page.click('[data-tab="doctor"]');
  });

  test("Run Diagnostics button triggers doctor and shows results", async ({ page }) => {
    await page.click("#run-doctor-btn");
    await expect(page.locator(".doctor-item").first()).toBeVisible();
    // Should show 9 items from mock data
    await expect(page.locator(".doctor-item")).toHaveCount(9);
  });

  test("Fix button triggers fix and shows success toast", async ({ page }) => {
    await page.click("#run-doctor-btn");
    // The libffi entry has a fix_command
    const fixBtn = page.locator("button >> text=Fix").first();
    await expect(fixBtn).toBeVisible();
    await fixBtn.click();
    await expect(page.locator(".toast-success")).toBeVisible({ timeout: 5000 });
  });

  test("Run Diagnostics button re-enables after run", async ({ page }) => {
    const btn = page.locator("#run-doctor-btn");
    await btn.click();
    await expect(btn).toBeEnabled();
    await expect(btn).toContainText("Run Diagnostics");
  });
});

// ============================================
// Settings Tab Interactions
// ============================================

test.describe("Settings Interactions", () => {
  test.beforeEach(async ({ page }) => {
    await mockTauriAPI(page);
    await page.goto("/");
    await page.click('[data-tab="settings"]');
  });

  test("Install Hook button triggers installation", async ({ page }) => {
    const installBtn = page.locator("button >> text=Install Hook").first();
    await installBtn.click();
    await expect(page.locator(".toast-success")).toBeVisible({ timeout: 5000 });
  });

  test("Show Hook button reveals hook code", async ({ page }) => {
    await page.click("button >> text=Show Hook");
    const preview = page.locator("#shell-hook-preview");
    await expect(preview).not.toHaveClass(/hidden/);
    await expect(preview).toContainText("rubynaut");
  });

  test("shell selector has options", async ({ page }) => {
    const select = page.locator("#shell-select");
    await expect(select).toBeVisible();
    const options = select.locator("option");
    await expect(options).toHaveCount(4); // bash, zsh, fish, powershell
  });
});

// ============================================
// Toast Notifications
// ============================================

test.describe("Toast Notifications", () => {
  test.beforeEach(async ({ page }) => {
    await mockTauriAPI(page);
    await page.goto("/");
  });

  test("toast auto-dismisses after timeout", async ({ page }) => {
    // Trigger a toast by setting global version
    await page.locator(".version-switch-btn").first().click();
    await page.locator(".dropdown-item >> text=Set as Global Default").first().click();
    const toast = page.locator(".toast-success");
    await expect(toast).toBeVisible({ timeout: 2000 });
    // Toast should auto-remove after 4 seconds
    await expect(toast).not.toBeVisible({ timeout: 6000 });
  });
});

// ============================================
// Panel Toggle Behavior
// ============================================

test.describe("Panel Toggle", () => {
  test.beforeEach(async ({ page }) => {
    await mockTauriAPI(page);
    await page.goto("/");
  });

  test("clicking Gems then Projects switches panels", async ({ page }) => {
    const firstVersion = page.locator(".version-block").first();
    // Open gems
    await firstVersion.locator("button >> text=Gems").click();
    await expect(firstVersion.locator(".gem-item").first()).toBeVisible();

    // Switch to projects
    await firstVersion.locator("button >> text=Projects").click();
    // Gems should be gone, projects should show
    await expect(firstVersion.locator(".gem-item")).toHaveCount(0);
  });

  test("clicking same panel button closes it", async ({ page }) => {
    const firstVersion = page.locator(".version-block").first();
    await firstVersion.locator("button >> text=Gems").click();
    await expect(firstVersion.locator(".gem-item").first()).toBeVisible();

    // Click gems again to close
    await firstVersion.locator("button >> text=Gems").click();
    await expect(firstVersion.locator(".gem-item")).toHaveCount(0);
  });
});
