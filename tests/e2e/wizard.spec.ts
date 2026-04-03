import { test, expect, Page } from "@playwright/test";

/**
 * Mock that simulates first launch (no Ruby installed, wizard not completed).
 */
async function mockFirstLaunch(page: Page) {
  await page.addInitScript(() => {
    // @ts-ignore
    window.__TAURI__ = {
      core: {
        invoke: async (cmd: string, args?: any) => {
          switch (cmd) {
            case "detect_platform":
              return { os: "macos", arch: "aarch64", package_manager: "homebrew", shell: "/bin/zsh", is_wsl: false, has_c_compiler: true, existing_ruby_managers: [] };
            case "get_installed_rubies":
              return [];
            case "get_available_rubies":
              return [
                { version: "4.0.2", installed: false, active: false, path: null, prebuilt_available: true },
                { version: "3.3.6", installed: false, active: false, path: null, prebuilt_available: true },
              ];
            case "get_active_version":
              return null;
            case "get_config":
              return { global_version: null, projects: [], wizard_completed: false };
            case "set_wizard_completed":
              return null;
            case "get_latest_stable_version":
              return "4.0.2";
            case "install_ruby":
              return null;
            case "set_global_version":
              return null;
            case "check_shell_hook":
              return [
                { shell: "zsh", rc_file: "/Users/test/.zshrc", installed: false },
                { shell: "bash", rc_file: "/Users/test/.bashrc", installed: false },
              ];
            case "install_shell_hook":
              return "Hook installed to /Users/test/.zshrc";
            case "install_gem":
              return "Successfully installed rails";
            case "get_gems_for_version":
              return [];
            case "run_doctor":
              return [];
            case "get_shell_hook":
              return "# hook";
            case "get_tracked_projects":
              return [];
            default:
              return null;
          }
        },
      },
      event: {
        listen: async (event: string, handler: Function) => {
          // Simulate instant install progress
          if (event === "install-progress") {
            setTimeout(() => handler({ payload: { stage: "done", percent: 100, message: "Installed" } }), 100);
          }
          return () => {};
        },
      },
      shell: { open: async () => {} },
      dialog: { open: async () => null },
    };
  });
}

/**
 * Mock that simulates returning user (wizard completed).
 */
async function mockReturningUser(page: Page) {
  await page.addInitScript(() => {
    // @ts-ignore
    window.__TAURI__ = {
      core: {
        invoke: async (cmd: string) => {
          switch (cmd) {
            case "detect_platform":
              return { os: "macos", arch: "aarch64", package_manager: "homebrew", shell: "/bin/zsh", is_wsl: false, has_c_compiler: true, existing_ruby_managers: [] };
            case "get_installed_rubies":
              return [{ version: "4.0.2", installed: true, active: true, path: "/Users/test/.rubies/4.0.2", prebuilt_available: true }];
            case "get_available_rubies":
              return [{ version: "4.0.2", installed: true, active: true, path: "/Users/test/.rubies/4.0.2", prebuilt_available: true }];
            case "get_active_version":
              return "4.0.2";
            case "get_config":
              return { global_version: "4.0.2", projects: [], wizard_completed: true };
            case "get_tracked_projects":
              return [];
            case "check_shell_hook":
              return [{ shell: "zsh", rc_file: "/Users/test/.zshrc", installed: true }];
            case "get_shell_hook":
              return "# hook";
            default:
              return null;
          }
        },
      },
      event: { listen: async () => () => {} },
      shell: { open: async () => {} },
      dialog: { open: async () => null },
    };
  });
}

test.describe("Getting Started Wizard", () => {
  test("shows wizard on first launch with no Ruby", async ({ page }) => {
    await mockFirstLaunch(page);
    await page.goto("/");
    await expect(page.locator("#wizard-overlay")).toBeVisible({ timeout: 5000 });
  });

  test("wizard step 1 shows welcome message", async ({ page }) => {
    await mockFirstLaunch(page);
    await page.goto("/");
    await expect(page.locator(".wizard-step[data-step='1']")).toHaveClass(/active/);
    await expect(page.locator(".wizard-step[data-step='1'] h2")).toContainText("Welcome");
  });

  test("wizard shows step indicators", async ({ page }) => {
    await mockFirstLaunch(page);
    await page.goto("/");
    const dots = page.locator(".wizard-dot");
    await expect(dots).toHaveCount(5);
  });

  test("clicking Get Started advances to step 2", async ({ page }) => {
    await mockFirstLaunch(page);
    await page.goto("/");
    await page.click("[data-action='wizard-next']");
    await expect(page.locator(".wizard-step[data-step='2']")).toHaveClass(/active/);
    await expect(page.locator(".wizard-step[data-step='2'] h2")).toContainText("Install Ruby");
  });

  test("step 2 shows latest version", async ({ page }) => {
    await mockFirstLaunch(page);
    await page.goto("/");
    await page.click("[data-action='wizard-next']");
    await expect(page.locator("#wizard-install-version")).toContainText("4.0.2");
  });

  test("install button triggers install and advances", async ({ page }) => {
    await mockFirstLaunch(page);
    await page.goto("/");
    await page.click("[data-action='wizard-next']"); // to step 2
    await page.click("[data-action='wizard-install-ruby']");
    // Should auto-advance to step 3 after install
    await expect(page.locator(".wizard-step[data-step='3']")).toHaveClass(/active/, { timeout: 5000 });
    await expect(page.locator("#wizard-global-status")).toContainText("4.0.2");
  });

  test("wizard does NOT show when wizard_completed is true", async ({ page }) => {
    await mockReturningUser(page);
    await page.goto("/");
    await expect(page.locator("#wizard-overlay")).toHaveClass(/hidden/);
  });

  test("wizard does NOT show when Ruby is already installed", async ({ page }) => {
    await mockReturningUser(page);
    await page.goto("/");
    await expect(page.locator("#wizard-overlay")).toHaveClass(/hidden/);
    // Dashboard should be visible
    await expect(page.locator("#tab-dashboard")).toHaveClass(/active/);
  });
});

test.describe("What's Next Panel", () => {
  test("What's Next panel has all resource links", async ({ page }) => {
    await mockFirstLaunch(page);
    await page.goto("/");
    // Navigate through wizard to completion step
    await page.click("[data-action='wizard-next']"); // step 1 -> 2
    await expect(page.locator(".wizard-step[data-step='2']")).toHaveClass(/active/);
    await page.click("[data-action='wizard-install-ruby']"); // install -> step 3
    await expect(page.locator(".wizard-step[data-step='3']")).toHaveClass(/active/, { timeout: 10000 });
    await page.locator(".wizard-step.active [data-action='wizard-next']").click(); // step 3 -> 4
    await expect(page.locator(".wizard-step[data-step='4']")).toHaveClass(/active/);
    await page.locator(".wizard-step.active [data-action='wizard-skip-shell']").click(); // step 4 -> 5
    await expect(page.locator(".wizard-step[data-step='5']")).toHaveClass(/active/);

    // Verify What's Next panel on wizard step 5
    await expect(page.locator("#wizard-whats-next .whats-next-card")).toHaveCount(4);
    await expect(page.locator("#wizard-whats-next")).toContainText("Rails Getting Started");
    await expect(page.locator("#wizard-whats-next")).toContainText("Ruby Koans");
    await expect(page.locator("#wizard-whats-next")).toContainText("Exercism");
    await expect(page.locator("#wizard-whats-next")).toContainText("Ruby Documentation");
  });

  test("What's Next panel has copy button", async ({ page }) => {
    await mockFirstLaunch(page);
    await page.goto("/");
    await page.click("[data-action='wizard-next']");
    await expect(page.locator(".wizard-step[data-step='2']")).toHaveClass(/active/);
    await page.click("[data-action='wizard-install-ruby']");
    await expect(page.locator(".wizard-step[data-step='3']")).toHaveClass(/active/, { timeout: 10000 });
    await page.locator(".wizard-step.active [data-action='wizard-next']").click();
    await expect(page.locator(".wizard-step[data-step='4']")).toHaveClass(/active/);
    await page.locator(".wizard-step.active [data-action='wizard-skip-shell']").click();
    await expect(page.locator(".wizard-step[data-step='5']")).toHaveClass(/active/);

    await expect(page.locator("[data-action='copy-hello-ruby']")).toBeVisible();
    await expect(page.locator(".whats-next-code")).toContainText("puts");
  });
});
