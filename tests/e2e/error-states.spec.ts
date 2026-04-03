import { test, expect, Page } from "@playwright/test";

/**
 * Mock Tauri API that simulates error conditions.
 */
async function mockTauriWithErrors(page: Page) {
  await page.addInitScript(() => {
    // @ts-ignore
    window.__TAURI__ = {
      core: {
        invoke: async (cmd: string, args?: any) => {
          switch (cmd) {
            case "detect_platform":
              return {
                os: "macos",
                arch: "aarch64",
                package_manager: "homebrew",
                shell: "/bin/zsh",
                is_wsl: false,
                has_c_compiler: true,
                existing_ruby_managers: ["rbenv", "rvm"],
              };
            case "get_installed_rubies":
              return [];
            case "get_available_rubies":
              return [
                { version: "4.0.2", installed: false, active: false, path: null, prebuilt_available: true },
                { version: "3.3.6", installed: false, active: false, path: null, prebuilt_available: true },
              ];
            case "get_active_version":
              return null;
            case "install_ruby":
              throw "Download failed: HTTP 404 — prebuilt binary may not exist for this platform";
            case "install_gem":
              throw "Failed to install gem: network error";
            case "uninstall_gem":
              throw "Cannot uninstall default gem";
            case "run_doctor":
              throw "Diagnostics failed: permission denied";
            case "run_doctor_fix":
              throw "Fix failed: command not found";
            case "bundle_install":
              throw "No Gemfile found in this project";
            case "get_gems_for_version":
              throw "Ruby 4.0.2 is not installed";
            case "check_shell_hook":
              return [];
            case "install_shell_hook":
              throw "Failed to write /Users/test/.zshrc: permission denied";
            case "get_tracked_projects":
              return [];
            case "get_shell_hook":
              return "# hook code";
            case "scan_project":
              return { path: "/test", project_name: "test", detected_version: null, source: null, version_installed: false, has_gemfile: false, has_gemfile_lock: false, folder_exists: true };
            default:
              return null;
          }
        },
      },
      event: {
        listen: async () => () => {},
      },
      shell: {
        open: async () => {},
      },
      dialog: {
        open: async () => "/Users/test/myapp",
      },
    };
  });
}

// ============================================
// Empty State (no rubies installed)
// ============================================

test.describe("Empty State", () => {
  test.beforeEach(async ({ page }) => {
    await mockTauriWithErrors(page);
    await page.goto("/");
  });

  test("dashboard shows zero installed count", async ({ page }) => {
    await expect(page.locator("#stat-installed")).toContainText("0");
  });

  test("dashboard shows dash for active version", async ({ page }) => {
    await expect(page.locator("#stat-active")).toContainText("—");
  });

  test("dashboard shows empty state message", async ({ page }) => {
    await expect(page.locator(".empty-state")).toBeVisible();
    await expect(page.locator(".empty-state")).toContainText("No Ruby versions installed");
  });

  test("empty state has Install Ruby button", async ({ page }) => {
    const installBtn = page.locator(".empty-state .btn-primary");
    await expect(installBtn).toContainText("Install Ruby");
  });

  test("empty state Install Ruby button navigates to install tab", async ({ page }) => {
    await page.locator(".empty-state .btn-primary").click();
    await expect(page.locator("#tab-install")).toHaveClass(/active/);
  });

  test("shows other Ruby managers detected toast", async ({ page }) => {
    await expect(page.locator(".toast-info")).toBeVisible({ timeout: 5000 });
    await expect(page.locator(".toast-info")).toContainText("rbenv");
  });
});

// ============================================
// Install Error Handling
// ============================================

test.describe("Install Errors", () => {
  test.beforeEach(async ({ page }) => {
    await mockTauriWithErrors(page);
    await page.goto("/");
    await page.click('[data-tab="install"]');
  });

  test("failed install shows error toast", async ({ page }) => {
    const installBtn = page.locator(".version-card .btn-primary >> text=Install").first();
    await installBtn.click();
    await expect(page.locator(".toast-error")).toBeVisible({ timeout: 5000 });
    await expect(page.locator(".toast-error")).toContainText("Install failed");
  });
});

// ============================================
// Doctor Error Handling
// ============================================

test.describe("Doctor Errors", () => {
  test.beforeEach(async ({ page }) => {
    await mockTauriWithErrors(page);
    await page.goto("/");
    await page.click('[data-tab="doctor"]');
  });

  test("doctor failure shows error message", async ({ page }) => {
    await page.click("#run-doctor-btn");
    const doctorResults = page.locator("#doctor-results .empty-state");
    await expect(doctorResults).toBeVisible({ timeout: 5000 });
    await expect(doctorResults).toContainText("Diagnostics failed");
  });

  test("button re-enables after error", async ({ page }) => {
    await page.click("#run-doctor-btn");
    const btn = page.locator("#run-doctor-btn");
    await expect(btn).toBeEnabled({ timeout: 5000 });
    await expect(btn).toContainText("Run Diagnostics");
  });
});

// ============================================
// Settings Error Handling
// ============================================

test.describe("Settings Errors", () => {
  test.beforeEach(async ({ page }) => {
    await mockTauriWithErrors(page);
    await page.goto("/");
    await page.click('[data-tab="settings"]');
  });

  test("empty shell hook status when no shells detected", async ({ page }) => {
    const container = page.locator("#shell-hook-status");
    // With empty array returned, container should have no items
    await expect(container.locator(".shell-hook-item")).toHaveCount(0);
  });
});

// ============================================
// Projects Empty State
// ============================================

test.describe("Projects Empty State", () => {
  test.beforeEach(async ({ page }) => {
    await mockTauriWithErrors(page);
    await page.goto("/");
    await page.click('[data-tab="projects"]');
  });

  test("no tracked projects section when list is empty", async ({ page }) => {
    await expect(page.locator("#tracked-projects-section")).toHaveClass(/hidden/);
  });

  test("Open Project Folder button still visible", async ({ page }) => {
    await expect(page.locator("button >> text=Open Project Folder")).toBeVisible();
  });
});
