import { Page } from "@playwright/test";

/**
 * Mock the Tauri IPC layer so the frontend works outside the Tauri webview.
 * Each invoke command returns realistic test data.
 */
export async function mockTauriAPI(page: Page) {
  await page.addInitScript(() => {
    const mockData = {
      installedRubies: [
        {
          version: "4.0.2",
          installed: true,
          active: true,
          path: "/Users/test/.rubies/4.0.2",
          prebuilt_available: true,
        },
        {
          version: "3.3.6",
          installed: true,
          active: false,
          path: "/Users/test/.rubies/3.3.6",
          prebuilt_available: true,
        },
      ],
      availableRubies: [
        {
          version: "4.0.2",
          installed: true,
          active: true,
          path: "/Users/test/.rubies/4.0.2",
          prebuilt_available: true,
        },
        {
          version: "3.3.6",
          installed: true,
          active: false,
          path: "/Users/test/.rubies/3.3.6",
          prebuilt_available: true,
        },
        {
          version: "3.2.4",
          installed: false,
          active: false,
          path: null,
          prebuilt_available: true,
        },
        {
          version: "3.1.6",
          installed: false,
          active: false,
          path: null,
          prebuilt_available: true,
        },
      ],
      platformInfo: {
        os: "macos",
        arch: "aarch64",
        package_manager: "homebrew",
        shell: "/bin/zsh",
        is_wsl: false,
        has_c_compiler: true,
        existing_ruby_managers: [],
      },
      doctorResults: [
        {
          name: "Rubies directory",
          status: "ok",
          message: "Found at /Users/test/.rubies",
          fix_hint: null,
          fix_command: null,
        },
        {
          name: "Installed Rubies",
          status: "ok",
          message: "2 version(s) installed",
          fix_hint: null,
          fix_command: null,
        },
        {
          name: "Shell integration",
          status: "ok",
          message: "Hook found in /Users/test/.zshrc",
          fix_hint: null,
          fix_command: null,
        },
        {
          name: "PATH resolution",
          status: "ok",
          message: "ruby resolves to /Users/test/.rubies/4.0.2/bin/ruby",
          fix_hint: null,
          fix_command: null,
        },
        {
          name: "Conflicting managers",
          status: "ok",
          message: "No other Ruby version managers detected",
          fix_hint: null,
          fix_command: null,
        },
        {
          name: "C compiler",
          status: "ok",
          message: "Available (needed for native gem extensions)",
          fix_hint: null,
          fix_command: null,
        },
        {
          name: "libyaml (YAML parsing)",
          status: "ok",
          message: "Found",
          fix_hint: null,
          fix_command: null,
        },
        {
          name: "OpenSSL (TLS/SSL)",
          status: "ok",
          message: "Found",
          fix_hint: null,
          fix_command: null,
        },
        {
          name: "libffi (Foreign function interface)",
          status: "error",
          message: "Not found — required by Ruby's native extensions",
          fix_hint: "Install via Homebrew: brew install libffi",
          fix_command: "brew install libffi",
        },
      ],
      gems: [
        { name: "bundler", version: "2.5.0", is_default: true },
        { name: "json", version: "2.7.0", is_default: true },
        { name: "rake", version: "13.1.0", is_default: true },
        { name: "rails", version: "7.2.0", is_default: false },
        { name: "nokogiri", version: "1.16.0", is_default: false },
      ],
      shellHookStatuses: [
        {
          shell: "zsh",
          rc_file: "/Users/test/.zshrc",
          installed: true,
        },
        {
          shell: "bash",
          rc_file: "/Users/test/.bashrc",
          installed: false,
        },
        {
          shell: "fish",
          rc_file: "/Users/test/.config/fish/conf.d/rubynaut.fish",
          installed: false,
        },
      ],
      trackedProjects: [
        {
          path: "/Users/test/myapp",
          project_name: "myapp",
          detected_version: "4.0.2",
          source: ".ruby-version",
          version_installed: true,
          has_gemfile: true,
          has_gemfile_lock: true,
          folder_exists: true,
        },
        {
          path: "/Users/test/legacy",
          project_name: "legacy",
          detected_version: "2.7.8",
          source: "Gemfile",
          version_installed: false,
          has_gemfile: true,
          has_gemfile_lock: false,
          folder_exists: true,
        },
      ],
      projectGems: [
        { name: "rails", version: "7.1.0" },
        { name: "puma", version: "6.4.0" },
        { name: "nokogiri", version: "1.16.0" },
      ],
    };

    // @ts-ignore
    window.__TAURI__ = {
      core: {
        invoke: async (cmd: string, args?: any) => {
          switch (cmd) {
            case "detect_platform":
              return mockData.platformInfo;
            case "get_installed_rubies":
              return mockData.installedRubies;
            case "get_available_rubies":
              return mockData.availableRubies;
            case "get_active_version":
              return "4.0.2";
            case "set_global_version":
              return null;
            case "set_local_version":
              return null;
            case "uninstall_ruby":
              return null;
            case "get_gems_for_version":
              return mockData.gems;
            case "install_gem":
              return "Successfully installed test-gem-1.0.0";
            case "uninstall_gem":
              return "Successfully uninstalled test-gem";
            case "run_doctor":
              return mockData.doctorResults;
            case "run_doctor_fix":
              return "Fixed successfully";
            case "get_shell_hook":
              return "# Rubynaut shell integration\nrubynaut_switch() { ... }";
            case "check_shell_hook":
              return mockData.shellHookStatuses;
            case "install_shell_hook":
              return "Hook installed to /Users/test/.zshrc";
            case "get_tracked_projects":
              return mockData.trackedProjects;
            case "add_tracked_project":
              return mockData.trackedProjects;
            case "remove_tracked_project":
              return [];
            case "scan_project":
              return mockData.trackedProjects[0];
            case "pick_folder":
              return "/Users/test/myapp";
            case "get_project_gems":
              return mockData.projectGems;
            case "bundle_install":
              return "Bundle complete! 42 gems installed.";
            case "install_ruby":
              return null;
            default:
              console.warn(`Unmocked Tauri command: ${cmd}`, args);
              return null;
          }
        },
      },
      event: {
        listen: async (event: string, handler: Function) => {
          // No-op for tests — events are not emitted
          return () => {};
        },
      },
      shell: {
        open: async (url: string) => {},
      },
      dialog: {
        open: async () => "/Users/test/myapp",
      },
    };
  });
}
