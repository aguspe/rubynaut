import { test, expect } from "@playwright/test";
import { mockTauriStateful } from "./helpers/tauri-mock-stateful";

test.describe("State Transitions", () => {
  test.beforeEach(async ({ page }) => {
    await mockTauriStateful(page);
    await page.goto("/");
  });

  test("install Ruby updates the installed list on dashboard", async ({ page }) => {
    // Initially 2 installed versions
    await expect(page.locator("#stat-installed")).toContainText("2");

    // Go to install tab and install 3.2.4
    await page.click('[data-tab="install"]');
    const card = page.locator(".version-card").filter({ hasText: "3.2.4" });
    await card.locator("[data-action='install-ruby']").click();
    await expect(page.locator(".toast-success")).toBeVisible({ timeout: 5000 });

    // Go back to dashboard — should now show 3 installed
    await page.click('[data-tab="dashboard"]');
    await expect(page.locator("#stat-installed")).toContainText("3");
  });

  test("install Ruby changes card to Installed on install tab", async ({ page }) => {
    await page.click('[data-tab="install"]');

    // 3.2.4 should have an Install button
    const card = page.locator(".version-card").filter({ hasText: "3.2.4" });
    await expect(card.locator("[data-action='install-ruby']")).toBeVisible();

    // Install it
    await card.locator("[data-action='install-ruby']").click();
    await expect(page.locator(".toast-success")).toBeVisible({ timeout: 5000 });

    // Refresh install tab — 3.2.4 should now show as installed
    await page.click('[data-tab="dashboard"]');
    await page.click('[data-tab="install"]');
    const updatedCard = page.locator(".version-card").filter({ hasText: "3.2.4" });
    await expect(updatedCard.locator(".badge-installed")).toBeVisible();
  });

  test("uninstall Ruby removes from installed list", async ({ page }) => {
    // Initially 2 installed
    await expect(page.locator(".version-block")).toHaveCount(2);

    // Remove 3.3.6
    const block = page.locator(".version-block").filter({ hasText: "3.3.6" });
    await block.locator("[data-action='uninstall-ruby']").click();

    // Confirm dialog
    await page.click("#dialog-confirm");
    await expect(page.locator(".toast-success")).toBeVisible({ timeout: 5000 });

    // Dashboard should refresh — only 1 installed now
    await expect(page.locator("#stat-installed")).toContainText("1");
  });

  test("set global version updates active badge", async ({ page }) => {
    // Initially 4.0.2 is active
    const firstBlock = page.locator(".version-block").first();
    await expect(firstBlock.locator(".badge-active")).toBeVisible();

    // Set 3.3.6 as global
    const secondBlock = page.locator(".version-block").nth(1);
    await secondBlock.locator("[data-action='toggle-version-menu']").click();
    await secondBlock.locator("[data-action='set-global']").click();
    await expect(page.locator(".toast-success")).toBeVisible({ timeout: 5000 });

    // After refresh, 3.3.6 should be active
    await expect(page.locator("#stat-active")).toContainText("3.3.6");
  });

  test("remove project updates tracked list", async ({ page }) => {
    await page.click('[data-tab="projects"]');

    // Initially 2 tracked projects
    await expect(page.locator(".tracked-project-item")).toHaveCount(2);

    // Remove the first project
    const removeBtn = page.locator("[data-action='remove-tracked-project']").first();
    await removeBtn.click();
    await expect(page.locator(".toast-success")).toBeVisible({ timeout: 5000 });

    // Should now have 1 project
    await expect(page.locator(".tracked-project-item")).toHaveCount(1);
  });

  test("install gem appears in gem list after refresh", async ({ page }) => {
    // Open gems panel for first version
    const firstBlock = page.locator(".version-block").first();
    await firstBlock.locator("[data-action='toggle-panel'][data-panel='gems']").click();
    await expect(page.locator(".gem-item").first()).toBeVisible();

    // Count initial gems
    const initialCount = await page.locator(".gem-item").count();

    // Install a new gem
    await page.fill("#gem-install-input", "puma");
    await page.click("[data-action='install-gem']");
    await expect(page.locator(".toast-success")).toBeVisible({ timeout: 5000 });

    // Panel refreshes — should have one more gem
    await expect(page.locator(".gem-item")).toHaveCount(initialCount + 1);
  });

  test("uninstall gem removes from gem list", async ({ page }) => {
    // Open gems panel
    const firstBlock = page.locator(".version-block").first();
    await firstBlock.locator("[data-action='toggle-panel'][data-panel='gems']").click();
    await expect(page.locator(".gem-item").first()).toBeVisible();

    const initialCount = await page.locator(".gem-item").count();

    // Uninstall a user gem (click remove button, confirm)
    await page.locator("[data-action='remove-gem']").first().click();
    await page.click("#dialog-confirm");
    await expect(page.locator(".toast-success")).toBeVisible({ timeout: 5000 });

    // Should have one fewer gem
    await expect(page.locator(".gem-item")).toHaveCount(initialCount - 1);
  });
});
