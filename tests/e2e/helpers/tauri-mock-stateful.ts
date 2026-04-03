import { Page } from "@playwright/test";

/**
 * Stateful mock of the Tauri IPC layer.
 * State mutates in response to actions (install, uninstall, set-global, etc.)
 * so tests can verify UI updates after user interactions.
 */
export async function mockTauriStateful(page: Page) {
  await page.addInitScript(() => {
    const store = {
      installedRubies: [
        { version: "4.0.2", installed: true, active: true, path: "/Users/test/.rubies/4.0.2", prebuilt_available: true },
        { version: "3.3.6", installed: true, active: false, path: "/Users/test/.rubies/3.3.6", prebuilt_available: true },
      ],
      availableRubies: [
        { version: "4.0.2", installed: true, active: true, path: "/Users/test/.rubies/4.0.2", prebuilt_available: true },
        { version: "3.3.6", installed: true, active: false, path: "/Users/test/.rubies/3.3.6", prebuilt_available: true },
        { version: "3.2.4", installed: false, active: false, path: null, prebuilt_available: true },
        { version: "3.1.6", installed: false, active: false, path: null, prebuilt_available: true },
      ],
      globalVersion: "4.0.2",
      gems: [
        { name: "bundler", version: "2.5.0", is_default: true },
        { name: "json", version: "2.7.0", is_default: true },
        { name: "rake", version: "13.1.0", is_default: true },
        { name: "rails", version: "7.2.0", is_default: false },
        { name: "nokogiri", version: "1.16.0", is_default: false },
      ],
      trackedProjects: [
        {
          path: "/Users/test/myapp", project_name: "myapp",
          detected_version: "4.0.2", source: ".ruby-version",
          version_installed: true, has_gemfile: true, has_gemfile_lock: true, folder_exists: true,
        },
        {
          path: "/Users/test/legacy", project_name: "legacy",
          detected_version: "2.7.8", source: "Gemfile",
          version_installed: false, has_gemfile: true, has_gemfile_lock: false, folder_exists: true,
        },
      ],
      shellHookStatuses: [
        { shell: "zsh", rc_file: "/Users/test/.zshrc", installed: true },
        { shell: "bash", rc_file: "/Users/test/.bashrc", installed: false },
        { shell: "fish", rc_file: "/Users/test/.config/fish/conf.d/rubynaut.fish", installed: false },
      ],
    };

    // @ts-ignore
    window.__TAURI__ = {
      core: {
        invoke: async (cmd: string, args?: any) => {
          switch (cmd) {
            case "detect_platform":
              return { os: "macos", arch: "aarch64", package_manager: "homebrew", shell: "/bin/zsh", is_wsl: false, has_c_compiler: true, existing_ruby_managers: [] };

            case "get_installed_rubies":
              return store.installedRubies;

            case "get_available_rubies":
              return store.availableRubies;

            case "get_active_version":
              return store.globalVersion;

            case "set_global_version": {
              const ver = args.version;
              store.globalVersion = ver;
              store.installedRubies.forEach((r: any) => { r.active = (r.version === ver); });
              store.availableRubies.forEach((r: any) => { r.active = (r.version === ver); });
              return null;
            }

            case "install_ruby": {
              const ver = args.version;
              const avail = store.availableRubies.find((r: any) => r.version === ver);
              if (avail) {
                avail.installed = true;
                avail.path = `/Users/test/.rubies/${ver}`;
              }
              if (!store.installedRubies.find((r: any) => r.version === ver)) {
                store.installedRubies.push({ version: ver, installed: true, active: false, path: `/Users/test/.rubies/${ver}`, prebuilt_available: true });
              }
              return null;
            }

            case "uninstall_ruby": {
              const ver = args.version;
              store.installedRubies = store.installedRubies.filter((r: any) => r.version !== ver);
              const avail = store.availableRubies.find((r: any) => r.version === ver);
              if (avail) { avail.installed = false; avail.active = false; avail.path = null; }
              if (store.globalVersion === ver) store.globalVersion = null;
              return null;
            }

            case "set_local_version":
              return null;

            case "get_gems_for_version":
              return store.gems;

            case "install_gem": {
              const name = args.gemName;
              const ver = args.gemVersion || "1.0.0";
              if (!store.gems.find((g: any) => g.name === name)) {
                store.gems.push({ name, version: ver, is_default: false });
              }
              return `Successfully installed ${name}-${ver}`;
            }

            case "uninstall_gem": {
              const name = args.gemName;
              store.gems = store.gems.filter((g: any) => g.name !== name);
              return `Successfully uninstalled ${name}`;
            }

            case "run_doctor":
              return [
                { name: "Rubies directory", status: "ok", message: "Found", fix_hint: null, fix_command: null },
                { name: "Installed Rubies", status: "ok", message: `${store.installedRubies.length} installed`, fix_hint: null, fix_command: null },
              ];

            case "run_doctor_fix":
              return "Fixed";

            case "check_shell_hook":
              return store.shellHookStatuses;

            case "install_shell_hook":
              return "Hook installed";

            case "get_shell_hook":
              return "# hook code";

            case "get_tracked_projects":
              return store.trackedProjects;

            case "add_tracked_project":
              return store.trackedProjects;

            case "remove_tracked_project": {
              const path = args.path;
              store.trackedProjects = store.trackedProjects.filter((p: any) => p.path !== path);
              return store.trackedProjects;
            }

            case "scan_project":
              return store.trackedProjects[0];

            case "pick_folder":
              return "/Users/test/myapp";

            case "get_project_gems":
              return [{ name: "rails", version: "7.1.0" }, { name: "puma", version: "6.4.0" }];

            case "bundle_install":
              return "Bundle complete!";

            default:
              console.warn(`Unmocked: ${cmd}`, args);
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
