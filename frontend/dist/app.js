// ============================================
// Rubynaut — Frontend Application
// ============================================

// Tauri IPC — accessed lazily since __TAURI__ is injected async
function invoke(...args) {
  return window.__TAURI__.core.invoke(...args);
}
function listen(...args) {
  return window.__TAURI__.event.listen(...args);
}

// HTML escaping to prevent XSS via user-controlled data
function escapeHtml(str) {
  if (str == null) return '';
  return String(str)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');
}

// ============================================
// Navigation
// ============================================

function switchTab(tabName) {
  document.querySelectorAll('.tab-content').forEach(el => el.classList.remove('active'));
  document.querySelectorAll('.nav-link').forEach(el => el.classList.remove('active'));

  document.getElementById(`tab-${tabName}`).classList.add('active');
  document.querySelector(`[data-tab="${tabName}"]`).classList.add('active');

  if (tabName === 'dashboard') refreshDashboard();
  if (tabName === 'install') refreshAvailable();
  if (tabName === 'projects') refreshTrackedProjects();
  if (tabName === 'settings') refreshShellHookStatus();
}

document.querySelectorAll('.nav-link').forEach(link => {
  link.addEventListener('click', (e) => {
    e.preventDefault();
    switchTab(link.dataset.tab);
  });
});

// ============================================
// Toast Notifications
// ============================================

function showToast(message, type = 'info') {
  const container = document.getElementById('toast-container');
  const toast = document.createElement('div');
  toast.className = `toast toast-${type}`;
  toast.textContent = message;
  container.appendChild(toast);
  setTimeout(() => toast.remove(), 4000);
}

// ============================================
// Confirm Dialog
// ============================================

function showConfirm(title, message) {
  return new Promise((resolve) => {
    const overlay = document.createElement('div');
    overlay.className = 'dialog-overlay';
    overlay.innerHTML = `
      <div class="dialog">
        <h3>${escapeHtml(title)}</h3>
        <p>${escapeHtml(message)}</p>
        <div class="dialog-actions">
          <button class="btn btn-secondary" id="dialog-cancel">Cancel</button>
          <button class="btn btn-danger" id="dialog-confirm">Confirm</button>
        </div>
      </div>
    `;
    document.body.appendChild(overlay);

    overlay.querySelector('#dialog-cancel').onclick = () => {
      overlay.remove();
      resolve(false);
    };
    overlay.querySelector('#dialog-confirm').onclick = () => {
      overlay.remove();
      resolve(true);
    };
  });
}

// ============================================
// Platform Detection
// ============================================

async function detectPlatform() {
  try {
    const info = await invoke('detect_platform');
    const badge = document.getElementById('platform-badge');
    const osName = { macos: 'macOS', linux: 'Linux', windows: 'Windows' }[info.os] || info.os;
    const archName = { aarch64: 'ARM64', x86_64: 'x64' }[info.arch] || info.arch;
    badge.textContent = `${osName} ${archName}`;

    if (info.existing_ruby_managers.length > 0) {
      showToast(`Other Ruby managers detected: ${info.existing_ruby_managers.join(', ')}. Check Doctor for details.`, 'info');
    }
  } catch (e) {
    console.error('Platform detection failed:', e);
  }
}

// ============================================
// Dashboard
// ============================================

// Track which panel is open per version: null, 'gems', or 'projects'
let openPanels = {};

async function refreshDashboard() {
  try {
    const [installed, active, available] = await Promise.all([
      invoke('get_installed_rubies'),
      invoke('get_active_version'),
      invoke('get_available_rubies'),
    ]);

    document.getElementById('stat-installed').textContent = installed.length;
    document.getElementById('stat-active').textContent = active || '—';
    document.getElementById('stat-available').textContent = available.filter(v => !v.installed).length;

    const list = document.getElementById('installed-list');

    if (installed.length === 0) {
      list.innerHTML = `
        <div class="empty-state">
          <div class="empty-gem"><img src="icons/rubynaut-logo.svg" alt=""></div>
          <p>No Ruby versions installed yet</p>
          <button class="btn btn-primary" data-action="switch-tab" data-tab="install">Install Ruby</button>
        </div>
      `;
      return;
    }

    list.innerHTML = installed.map(ruby => {
      const v = escapeHtml(ruby.version);
      return `
      <div class="version-block" id="version-block-${v}">
        <div class="version-item ${ruby.active ? 'active-version' : ''}">
          <div class="version-info">
            <span class="version-number">${v}</span>
            <div class="version-badges">
              ${ruby.active ? '<span class="version-badge badge-active">Active</span>' : ''}
              ${ruby.version === active ? '<span class="version-badge badge-global">Global</span>' : ''}
            </div>
          </div>
          <div class="version-actions">
            <button class="btn btn-small btn-ghost ${openPanels[ruby.version] === 'gems' ? 'btn-ghost-active' : ''}" data-action="toggle-panel" data-version="${v}" data-panel="gems">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="12,2 22,8.5 22,15.5 12,22 2,15.5 2,8.5"/></svg>
              Gems
            </button>
            <button class="btn btn-small btn-ghost ${openPanels[ruby.version] === 'projects' ? 'btn-ghost-active' : ''}" data-action="toggle-panel" data-version="${v}" data-panel="projects">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
              Projects
            </button>
            <div class="version-dropdown">
              <button class="btn btn-small btn-secondary version-switch-btn" data-action="toggle-version-menu" data-version="${v}">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><circle cx="12" cy="12" r="1"/><circle cx="12" cy="5" r="1"/><circle cx="12" cy="19" r="1"/></svg>
                Use
              </button>
              <div class="dropdown-menu hidden" id="menu-${v}">
                <button class="dropdown-item" data-action="set-global" data-version="${v}">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>
                  Set as Global Default
                  <span class="dropdown-hint">Used in all terminals</span>
                </button>
                <button class="dropdown-item" data-action="set-local" data-version="${v}">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
                  Set as Local (Project)
                  <span class="dropdown-hint">Writes .ruby-version file</span>
                </button>
              </div>
            </div>
            <button class="btn btn-small btn-danger" data-action="uninstall-ruby" data-version="${v}">Remove</button>
          </div>
        </div>
        <div class="inline-panel" id="panel-${v}"></div>
      </div>
    `;
    }).join('');

    // Re-open any panels that were open before refresh
    for (const [version, panelType] of Object.entries(openPanels)) {
      if (panelType) loadPanel(version, panelType);
    }
  } catch (e) {
    console.error('Dashboard refresh failed:', e);
    showToast('Failed to load dashboard', 'error');
  }
}

function togglePanel(version, panelType) {
  const current = openPanels[version];
  if (current === panelType) {
    // Close the panel
    openPanels[version] = null;
    const panel = document.getElementById(`panel-${version}`);
    if (panel) panel.innerHTML = '';
    // Update button states
    updatePanelButtons(version);
  } else {
    // Open the requested panel (closes any other)
    openPanels[version] = panelType;
    loadPanel(version, panelType);
    updatePanelButtons(version);
  }
}

function updatePanelButtons(version) {
  const block = document.getElementById(`version-block-${version}`);
  if (!block) return;
  block.querySelectorAll('.btn-ghost').forEach(btn => {
    btn.classList.remove('btn-ghost-active');
  });
  const activeType = openPanels[version];
  if (activeType) {
    // Find the button that matches
    const buttons = block.querySelectorAll('.version-actions > .btn-ghost');
    buttons.forEach(btn => {
      if (btn.textContent.trim().toLowerCase().includes(activeType)) {
        btn.classList.add('btn-ghost-active');
      }
    });
  }
}

async function loadPanel(version, panelType) {
  const panel = document.getElementById(`panel-${version}`);
  if (!panel) return;

  if (panelType === 'gems') {
    panel.innerHTML = '<div class="inline-panel-content"><div class="loading-state"><div class="spinner"></div><p>Loading gems...</p></div></div>';
    try {
      currentGemsVersion = version;
      currentGemsData = await invoke('get_gems_for_version', { version });
      renderInlineGems(panel, version, currentGemsData);
    } catch (e) {
      panel.innerHTML = `<div class="inline-panel-content"><p style="color:var(--slate-500)">Failed to load gems: ${escapeHtml(e)}</p></div>`;
    }
  } else if (panelType === 'projects') {
    panel.innerHTML = '<div class="inline-panel-content"><div class="loading-state"><div class="spinner"></div><p>Loading projects...</p></div></div>';
    try {
      const allProjects = await invoke('get_tracked_projects');
      const versionProjects = allProjects.filter(p => p.detected_version === version);
      renderInlineProjects(panel, version, versionProjects);
    } catch (e) {
      panel.innerHTML = `<div class="inline-panel-content"><p style="color:var(--slate-500)">Failed to load projects: ${escapeHtml(e)}</p></div>`;
    }
  }
}

function renderInlineGems(panel, version, gems) {
  const defaultCount = gems.filter(g => g.is_default).length;
  const userCount = gems.filter(g => !g.is_default).length;

  panel.innerHTML = `
    <div class="inline-panel-content">
      <div class="gems-install-row">
        <input type="text" id="gem-install-input" class="text-input" placeholder="Gem name...">
        <input type="text" id="gem-version-input" class="text-input gem-version-field" placeholder="Version (optional)">
        <button class="btn btn-primary btn-small" id="gem-install-btn" data-action="install-gem">Install</button>
      </div>
      <div class="gems-filters">
        <input type="text" id="gems-search" class="text-input" placeholder="Filter gems...">
        <label class="filter-label">
          <input type="checkbox" id="gems-show-default" checked> Default (${defaultCount})
        </label>
        <label class="filter-label">
          <input type="checkbox" id="gems-show-user" checked> User (${userCount})
        </label>
      </div>
      <div id="gems-list" class="gems-list">
        ${gems.length === 0
          ? '<div class="empty-state" style="grid-column:1/-1;padding:24px"><p>No gems found</p></div>'
          : gems.map(gem => `
            <div class="gem-item">
              <div>
                <span class="gem-name">${escapeHtml(gem.name)}</span>
                <span class="${gem.is_default ? 'gem-badge-default' : 'gem-badge-user'}">${gem.is_default ? 'default' : 'user'}</span>
              </div>
              <div class="gem-right">
                <span class="gem-version">${escapeHtml(gem.version)}</span>
                ${!gem.is_default ? `<button class="gem-remove-btn" data-action="remove-gem" data-gem="${escapeHtml(gem.name)}" title="Uninstall">&#10005;</button>` : ''}
              </div>
            </div>
          `).join('')
        }
      </div>
    </div>
  `;
}

function renderInlineProjects(panel, version, projects) {
  const folderSvg = '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>';

  if (projects.length === 0) {
    panel.innerHTML = `
      <div class="inline-panel-content">
        <div class="empty-state" style="padding:24px">
          <p>No projects using Ruby ${version}</p>
          <p style="font-size:12px;color:var(--slate-500);margin-top:4px">Open a project folder in the Projects tab to track it</p>
        </div>
      </div>
    `;
    return;
  }

  panel.innerHTML = `
    <div class="inline-panel-content">
      ${projects.map(p => {
        const shortPath = p.path.replace(/^\/Users\/[^/]+/, '~');
        return `
          <div class="tracked-project-item ${!p.folder_exists ? 'folder-missing' : ''}">
            <div class="tracked-project-info">
              <div class="tracked-project-icon">${folderSvg}</div>
              <div class="tracked-project-details">
                <div class="tracked-project-name">${escapeHtml(p.project_name)}</div>
                <div class="tracked-project-path">${escapeHtml(shortPath)}</div>
              </div>
            </div>
            <div class="tracked-project-actions">
              ${!p.folder_exists ? '<span class="version-badge badge-folder-gone">Missing</span>' : `<span class="version-badge badge-ready">via ${escapeHtml(p.source || '?')}</span>`}
              <button class="btn btn-small btn-ghost" data-action="remove-tracked-project" data-path="${escapeHtml(p.path.replace(/'/g, "\\'"))}">Remove</button>
            </div>
          </div>
        `;
      }).join('')}
    </div>
  `;
}

function toggleVersionMenu(version) {
  // Close all other menus first
  document.querySelectorAll('.dropdown-menu').forEach(m => {
    if (m.id !== `menu-${version}`) m.classList.add('hidden');
  });
  const menu = document.getElementById(`menu-${version}`);
  menu.classList.toggle('hidden');
}

// Close dropdown when clicking outside
document.addEventListener('click', (e) => {
  if (!e.target.closest('.version-dropdown')) {
    document.querySelectorAll('.dropdown-menu').forEach(m => m.classList.add('hidden'));
  }
});

async function setGlobal(version) {
  document.querySelectorAll('.dropdown-menu').forEach(m => m.classList.add('hidden'));
  try {
    await invoke('set_global_version', { version });
    showToast(`Ruby ${version} set as global default — active in all new terminals`, 'success');
    refreshDashboard();
  } catch (e) {
    showToast(e, 'error');
  }
}

async function promptSetLocal(version) {
  document.querySelectorAll('.dropdown-menu').forEach(m => m.classList.add('hidden'));

  try {
    const selected = await invoke('pick_folder', {
      title: `Select project folder for Ruby ${version}`
    });

    if (!selected) return; // User cancelled

    await invoke('set_local_version', { path: selected, version });
    const folderName = selected.split('/').pop() || selected;
    showToast(`Ruby ${version} pinned to ${folderName}/ via .ruby-version`, 'success');
  } catch (e) {
    showToast(`Failed: ${e}`, 'error');
  }
}

async function uninstallRuby(version) {
  const confirmed = await showConfirm(
    'Uninstall Ruby',
    `Remove Ruby ${version} and all its gems? This cannot be undone.`
  );
  if (!confirmed) return;

  try {
    await invoke('uninstall_ruby', { version });
    showToast(`Ruby ${version} removed`, 'success');
    refreshDashboard();
  } catch (e) {
    showToast(e, 'error');
  }
}

// ============================================
// Install
// ============================================

let installing = false;
let allAvailableVersions = [];
let currentEngineFilter = 'recommended';

function getEngine(version) {
  if (version.startsWith('jruby-')) return 'jruby';
  if (version.startsWith('truffleruby+graalvm-')) return 'truffleruby';
  if (version.startsWith('truffleruby-')) return 'truffleruby';
  return 'ruby';
}

function isRecommended(version, allVersions) {
  const engine = getEngine(version);
  if (engine !== 'ruby') return false;
  // Recommended: latest patch of each major.minor series (e.g. 4.0.x, 3.3.x, 3.2.x)
  const parts = version.split('.');
  if (parts.length < 2) return false;
  const series = parts[0] + '.' + parts[1];
  const firstInSeries = allVersions.find(v => getEngine(v.version) === 'ruby' && v.version.startsWith(series + '.'));
  return firstInSeries && firstInSeries.version === version;
}

function renderAvailableVersions() {
  const grid = document.getElementById('available-list');
  const search = (document.getElementById('install-search')?.value || '').toLowerCase();

  let filtered = allAvailableVersions;

  // Engine filter
  if (currentEngineFilter === 'recommended') {
    filtered = filtered.filter(v => isRecommended(v.version, allAvailableVersions));
  } else if (currentEngineFilter !== 'all') {
    filtered = filtered.filter(v => getEngine(v.version) === currentEngineFilter);
  }

  // Search filter
  if (search) {
    filtered = filtered.filter(v => v.version.toLowerCase().includes(search));
  }

  if (filtered.length === 0) {
    grid.innerHTML = `<div class="empty-state"><p>No versions match your filter</p></div>`;
    return;
  }

  // Find the latest CRuby for the "Recommended" badge
  const latestCRuby = allAvailableVersions.find(v => getEngine(v.version) === 'ruby');

  grid.innerHTML = filtered.map(ruby => {
    const v = escapeHtml(ruby.version);
    const engine = getEngine(ruby.version);
    const isLatest = latestCRuby && ruby.version === latestCRuby.version;

    let engineLabel = '';
    if (engine === 'jruby') engineLabel = '<span class="version-badge badge-engine">JRuby</span>';
    else if (engine === 'truffleruby') engineLabel = '<span class="version-badge badge-engine">TruffleRuby</span>';

    let statusBadge;
    if (ruby.installed) {
      statusBadge = '<span class="version-badge badge-installed">Installed</span>';
    } else if (isLatest) {
      statusBadge = '<span class="version-badge badge-recommended">Recommended</span>';
    } else {
      statusBadge = '<span class="version-badge badge-prebuilt">Pre-built</span>';
    }

    return `
      <div class="version-card ${ruby.installed ? 'installed' : ''} ${isLatest && !ruby.installed ? 'recommended' : ''}">
        <span class="version-number">${v}</span>
        <div class="version-card-badges">
          ${engineLabel}
          ${statusBadge}
        </div>
        ${!ruby.installed
          ? `<button class="btn ${isLatest ? 'btn-primary' : 'btn-secondary'} btn-small" data-action="install-ruby" data-version="${v}" ${installing ? 'disabled' : ''}>Install</button>`
          : '<button class="btn btn-ghost btn-small" disabled>Installed</button>'
        }
      </div>
    `;
  }).join('');
}

function filterEngine(engine) {
  currentEngineFilter = engine;
  // Update active tab
  document.querySelectorAll('.engine-tab').forEach(tab => {
    tab.classList.toggle('active', tab.dataset.engine === engine);
  });
  renderAvailableVersions();
}

async function refreshAvailable() {
  const grid = document.getElementById('available-list');
  grid.innerHTML = `
    <div class="loading-state">
      <div class="spinner"></div>
      <p>Loading available versions...</p>
    </div>
  `;

  try {
    allAvailableVersions = await invoke('get_available_rubies');
    renderAvailableVersions();
  } catch (e) {
    grid.innerHTML = `<div class="empty-state"><p>Failed to load versions: ${escapeHtml(e)}</p></div>`;
  }
}

async function installRuby(version) {
  if (installing) return;
  installing = true;

  const progressContainer = document.getElementById('install-progress-container');
  progressContainer.classList.remove('hidden');

  try {
    await invoke('install_ruby', { version });
    showToast(`Ruby ${version} installed successfully!`, 'success');
    refreshAvailable();
    refreshDashboard();
    // Show "What's Next" panel
    const whatsNext = document.getElementById('install-whats-next');
    if (whatsNext) {
      renderWhatsNextPanel(whatsNext);
      whatsNext.classList.remove('hidden');
    }
  } catch (e) {
    showToast(`Install failed: ${e}`, 'error');
  } finally {
    installing = false;
    setTimeout(() => progressContainer.classList.add('hidden'), 2000);
  }
}

// ============================================
// What's Next Panel
// ============================================

function renderWhatsNextPanel(container) {
  container.innerHTML = `
    <h3 class="whats-next-title">What's Next?</h3>
    <div class="whats-next-grid">
      <a class="whats-next-card" data-action="open-external" data-url="https://guides.rubyonrails.org/getting_started.html" href="#">
        <div class="whats-next-card-title">Rails Getting Started</div>
        <div class="whats-next-card-desc">Build your first web app with Ruby on Rails</div>
      </a>
      <a class="whats-next-card" data-action="open-external" data-url="https://www.rubykoans.com/" href="#">
        <div class="whats-next-card-title">Ruby Koans</div>
        <div class="whats-next-card-desc">Learn Ruby through test-driven exercises</div>
      </a>
      <a class="whats-next-card" data-action="open-external" data-url="https://exercism.org/tracks/ruby" href="#">
        <div class="whats-next-card-title">Exercism Ruby Track</div>
        <div class="whats-next-card-desc">Practice with mentored coding challenges</div>
      </a>
      <a class="whats-next-card" data-action="open-external" data-url="https://www.ruby-lang.org/en/documentation/" href="#">
        <div class="whats-next-card-title">Ruby Documentation</div>
        <div class="whats-next-card-desc">Official guides, tutorials, and API reference</div>
      </a>
    </div>
    <div class="whats-next-try">
      <p class="whats-next-try-label">Try Ruby right now:</p>
      <code class="whats-next-code">ruby -e "puts 'Hello, Ruby!'"</code>
      <button class="btn btn-secondary btn-small" data-action="copy-hello-ruby">Copy to Clipboard</button>
    </div>
  `;
}

function copyHelloRuby() {
  const cmd = 'ruby -e "puts \'Hello, Ruby!\'"';
  if (navigator.clipboard) {
    navigator.clipboard.writeText(cmd).then(() => {
      showToast('Copied to clipboard!', 'success');
    }).catch(() => {
      showToast('Could not copy — try selecting and copying manually', 'error');
    });
  }
}

// Install progress listener is set up in init()

// ============================================
// Projects
// ============================================

async function openProject() {
  try {
    const selected = await invoke('pick_folder', {
      title: 'Select a Ruby project folder'
    });

    if (!selected) return;

    // Scan and auto-register the project
    const result = await invoke('scan_project', { path: selected });
    renderProjectResult(result);

    // Add to tracked projects and refresh the list
    const tracked = await invoke('add_tracked_project', { path: selected });
    renderTrackedProjectsList(tracked);
  } catch (e) {
    showToast(`Failed to scan project: ${e}`, 'error');
  }
}

function renderProjectResult(scan) {
  const container = document.getElementById('project-result');
  container.classList.remove('hidden');

  const shortPath = escapeHtml(scan.path.replace(/^\/Users\/[^/]+/, '~'));
  const version = scan.detected_version;
  const escapedVersion = escapeHtml(version);
  const escapedSource = escapeHtml(scan.source);
  const escapedName = escapeHtml(scan.project_name);
  const escapedPath = escapeHtml(scan.path.replace(/'/g, "\\'"));

  let versionRow;
  let actionsHtml;

  if (version && scan.version_installed) {
    // Version detected and installed — all good
    versionRow = `
      <div class="project-detail-row success">
        <span class="project-detail-icon" style="color:var(--success)">&#10003;</span>
        <div class="project-detail-content">
          <div class="project-detail-label">Ruby ${escapedVersion}</div>
          <div class="project-detail-value">Detected from ${escapedSource} — already installed</div>
        </div>
        <span class="version-badge badge-installed">Ready</span>
      </div>
    `;
    actionsHtml = `
      <button class="btn btn-primary" data-action="set-global-from-project" data-version="${escapedVersion}">Set as Global</button>
    `;
  } else if (version && !scan.version_installed) {
    // Version detected but not installed — offer to install
    versionRow = `
      <div class="project-detail-row missing">
        <span class="project-detail-icon" style="color:var(--error)">&#10007;</span>
        <div class="project-detail-content">
          <div class="project-detail-label">Ruby ${escapedVersion}</div>
          <div class="project-detail-value">Detected from ${escapedSource} — not installed</div>
        </div>
        <span class="version-badge" style="background:var(--error-bg);color:var(--error);border:1px solid rgba(239,68,68,0.3)">Missing</span>
      </div>
    `;
    actionsHtml = `
      <button class="btn btn-primary" data-action="install-for-project" data-version="${escapedVersion}" data-path="${escapedPath}">
        Install Ruby ${escapedVersion} &amp; Set Up
      </button>
    `;
  } else {
    // No version detected
    versionRow = `
      <div class="project-detail-row warning">
        <span class="project-detail-icon" style="color:var(--warning)">&#9888;</span>
        <div class="project-detail-content">
          <div class="project-detail-label">No Ruby version specified</div>
          <div class="project-detail-value">No .ruby-version, .tool-versions, or Gemfile ruby constraint found</div>
        </div>
      </div>
    `;
    actionsHtml = `
      <button class="btn btn-secondary" data-action="switch-tab" data-tab="install">Browse Ruby Versions</button>
    `;
  }

  const gemfileRow = scan.has_gemfile ? `
    <div class="project-detail-row success">
      <span class="project-detail-icon" style="color:var(--success)">&#10003;</span>
      <div class="project-detail-content">
        <div class="project-detail-label">Gemfile found</div>
        <div class="project-detail-value">Run <code>bundle install</code> after Ruby is set up</div>
      </div>
    </div>
  ` : '';

  container.innerHTML = `
    <div class="project-card">
      <div class="project-card-header">
        <div class="project-card-icon">
          <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
          </svg>
        </div>
        <div>
          <div class="project-card-title">${escapedName}</div>
          <div class="project-card-path">${shortPath}</div>
        </div>
      </div>

      <div class="project-detail-rows">
        ${versionRow}
        ${gemfileRow}
      </div>

      <div class="project-actions">
        ${actionsHtml}
        <button class="btn btn-secondary" data-action="open-project">Scan Another</button>
      </div>
    </div>
  `;
}

async function installForProject(version, projectPath) {
  // Switch to install tab and trigger install, then come back to set local version
  showToast(`Installing Ruby ${version}...`, 'info');

  try {
    await invoke('install_ruby', { version });
    // After install, set it as local version for the project
    await invoke('set_local_version', { path: projectPath, version });
    showToast(`Ruby ${version} installed and pinned to project!`, 'success');
    // Re-scan to update the UI
    const result = await invoke('scan_project', { path: projectPath });
    renderProjectResult(result);
    refreshTrackedProjects();
    refreshDashboard();
  } catch (e) {
    showToast(`Failed: ${e}`, 'error');
  }
}

async function refreshTrackedProjects() {
  try {
    const projects = await invoke('get_tracked_projects');
    renderTrackedProjectsList(projects);
  } catch (e) {
    console.error('Failed to load tracked projects:', e);
  }
}

function renderTrackedProjectsList(projects) {
  const section = document.getElementById('tracked-projects-section');
  const list = document.getElementById('tracked-projects-list');

  if (!projects || projects.length === 0) {
    section.classList.add('hidden');
    return;
  }

  section.classList.remove('hidden');

  const folderSvg = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>';

  list.innerHTML = projects.map((p, idx) => {
    const shortPath = escapeHtml(p.path.replace(/^\/Users\/[^/]+/, '~'));
    let badge, statusClass;

    if (!p.folder_exists) {
      badge = '<span class="version-badge badge-folder-gone">Folder Missing</span>';
      statusClass = 'folder-missing';
    } else if (p.detected_version && p.version_installed) {
      badge = '<span class="version-badge badge-ready">Ready</span>';
      statusClass = '';
    } else if (p.detected_version && !p.version_installed) {
      badge = '<span class="version-badge badge-missing-ruby">Not Installed</span>';
      statusClass = '';
    } else {
      badge = '<span class="version-badge badge-no-version">No Version</span>';
      statusClass = '';
    }

    const versionDisplay = p.detected_version
      ? `<span class="tracked-project-version-number">${escapeHtml(p.detected_version)}</span>
         <span class="tracked-project-source">via ${escapeHtml(p.source || '?')}</span>`
      : '<span class="tracked-project-source">—</span>';

    const esc = escapeHtml(p.path.replace(/'/g, "\\'"));
    const escVersion = escapeHtml(p.detected_version);
    const installBtn = (p.detected_version && !p.version_installed && p.folder_exists)
      ? `<button class="btn btn-small btn-primary" data-action="install-for-project" data-version="${escVersion}" data-path="${esc}">Install</button>`
      : '';
    const bundleBtn = (p.has_gemfile && p.version_installed && p.folder_exists)
      ? `<button class="btn btn-small btn-secondary" id="bundle-btn-${idx}" data-action="run-bundle" data-version="${escVersion}" data-path="${esc}" data-idx="${idx}">Bundle</button>`
      : '';
    const gemsBtn = (p.has_gemfile_lock && p.folder_exists)
      ? `<button class="btn btn-small btn-ghost" data-action="toggle-project-gems" data-path="${esc}" data-idx="${idx}">Gems</button>`
      : '';

    return `
      <div class="tracked-project-block" id="project-block-${idx}">
        <div class="tracked-project-item ${statusClass}">
          <div class="tracked-project-info">
            <div class="tracked-project-icon">${folderSvg}</div>
            <div class="tracked-project-details">
              <div class="tracked-project-name">${escapeHtml(p.project_name)}</div>
              <div class="tracked-project-path">${shortPath}</div>
            </div>
          </div>
          <div class="tracked-project-version">
            ${versionDisplay}
            ${badge}
          </div>
          <div class="tracked-project-actions">
            ${gemsBtn}
            ${bundleBtn}
            ${installBtn}
            <button class="btn btn-small btn-ghost" data-action="remove-tracked-project" data-path="${esc}">Remove</button>
          </div>
        </div>
        <div class="project-inline-panel" id="project-panel-${idx}"></div>
      </div>
    `;
  }).join('');
}

let openProjectGems = {};

async function runBundleInstall(rubyVersion, projectPath, idx) {
  const btn = document.getElementById(`bundle-btn-${idx}`);
  const panel = document.getElementById(`project-panel-${idx}`);
  if (btn) { btn.disabled = true; btn.textContent = 'Installing...'; }

  panel.innerHTML = `
    <div class="inline-panel-content">
      <div class="bundle-log">
        <div class="bundle-log-header">
          <span>Bundle Install</span>
          <div class="bundle-spinner" id="bundle-spinner-${idx}">
            <div class="spinner" style="width:16px;height:16px;border-width:2px;margin:0"></div>
          </div>
        </div>
        <pre class="bundle-output" id="bundle-output-${idx}">Running bundle install...</pre>
      </div>
    </div>
  `;

  try {
    const result = await invoke('bundle_install', { rubyVersion, projectPath });
    document.getElementById(`bundle-output-${idx}`).textContent = result || 'Bundle install completed successfully.';
    showToast('Bundle install completed', 'success');
  } catch (e) {
    document.getElementById(`bundle-output-${idx}`).textContent = e;
    showToast('Bundle install failed', 'error');
  } finally {
    if (btn) { btn.disabled = false; btn.textContent = 'Bundle'; }
    // Always stop the spinner
    const spinner = document.getElementById(`bundle-spinner-${idx}`);
    if (spinner) spinner.innerHTML = '';
  }
}

async function toggleProjectGems(projectPath, idx) {
  const panel = document.getElementById(`project-panel-${idx}`);
  if (openProjectGems[idx]) {
    panel.innerHTML = '';
    openProjectGems[idx] = false;
    return;
  }

  openProjectGems[idx] = true;
  panel.innerHTML = '<div class="inline-panel-content"><div class="loading-state"><div class="spinner"></div><p>Loading gems...</p></div></div>';

  try {
    const gems = await invoke('get_project_gems', { projectPath });
    if (gems.length === 0) {
      panel.innerHTML = '<div class="inline-panel-content"><p style="color:var(--slate-500);padding:8px 0">No gems found in Gemfile.lock</p></div>';
      return;
    }
    panel.innerHTML = `
      <div class="inline-panel-content">
        <div class="project-gems-header">${gems.length} gems in Gemfile.lock</div>
        <div class="gems-list">
          ${gems.map(g => `
            <div class="gem-item">
              <span class="gem-name">${escapeHtml(g.name)}</span>
              <span class="gem-version">${escapeHtml(g.version)}</span>
            </div>
          `).join('')}
        </div>
      </div>
    `;
  } catch (e) {
    panel.innerHTML = `<div class="inline-panel-content"><p style="color:var(--slate-500)">${escapeHtml(e)}</p></div>`;
    openProjectGems[idx] = false;
  }
}

async function removeTrackedProject(path) {
  try {
    await invoke('remove_tracked_project', { path });
    showToast('Project removed from tracking', 'success');
    // Refresh any open projects panels and the tracked list on Projects tab
    for (const [version, panelType] of Object.entries(openPanels)) {
      if (panelType === 'projects') loadPanel(version, 'projects');
    }
    refreshTrackedProjects();
  } catch (e) {
    showToast(e, 'error');
  }
}

async function setGlobalFromProject(version) {
  try {
    await invoke('set_global_version', { version });
    showToast(`Ruby ${version} set as global default`, 'success');
  } catch (e) {
    showToast(e, 'error');
  }
}

// ============================================
// Doctor
// ============================================

async function runDoctor() {
  const btn = document.getElementById('run-doctor-btn');
  const results = document.getElementById('doctor-results');
  btn.disabled = true;
  btn.textContent = 'Running...';

  try {
    const diagnostics = await invoke('run_doctor');
    results.innerHTML = diagnostics.map((d, i) => {
      const icon = { ok: '&#10003;', warning: '&#9888;', error: '&#10007;' }[d.status];
      const fixBtn = d.fix_command
        ? `<button class="btn btn-small btn-primary" id="fix-btn-${i}" data-action="run-doctor-fix" data-command="${escapeHtml(d.fix_command)}" data-idx="${i}">Fix</button>`
        : '';
      return `
        <div class="doctor-item">
          <div class="doctor-icon ${escapeHtml(d.status)}">${icon}</div>
          <div class="doctor-body">
            <div class="doctor-name">${escapeHtml(d.name)}</div>
            <div class="doctor-message">${escapeHtml(d.message)}</div>
            ${d.fix_hint ? `<div class="doctor-hint">${escapeHtml(d.fix_hint)}</div>` : ''}
          </div>
          ${fixBtn}
        </div>
      `;
    }).join('');
  } catch (e) {
    results.innerHTML = `<div class="empty-state"><p>Diagnostics failed: ${escapeHtml(e)}</p></div>`;
  } finally {
    btn.disabled = false;
    btn.textContent = 'Run Diagnostics';
  }
}

async function runDoctorFix(command, idx) {
  const btn = document.getElementById(`fix-btn-${idx}`);
  if (btn) { btn.disabled = true; btn.textContent = 'Fixing...'; }

  try {
    await invoke('run_doctor_fix', { command });
    showToast(`Fixed: ${command}`, 'success');
    // Re-run diagnostics to update the results
    runDoctor();
  } catch (e) {
    showToast(`Fix failed: ${e}`, 'error');
    if (btn) { btn.disabled = false; btn.textContent = 'Fix'; }
  }
}

// ============================================
// Settings — Shell Hook
// ============================================

async function refreshShellHookStatus() {
  const container = document.getElementById('shell-hook-status');
  try {
    const statuses = await invoke('check_shell_hook');
    container.innerHTML = statuses.map(s => {
      const shortFile = escapeHtml(s.rc_file.replace(/^\/Users\/[^/]+/, '~'));
      return `
        <div class="shell-hook-item ${s.installed ? 'installed' : ''}">
          <div class="shell-hook-info">
            <span class="shell-hook-icon ${s.installed ? 'ok' : 'missing'}">${s.installed ? '&#10003;' : '&#9675;'}</span>
            <div>
              <div class="shell-hook-name">${escapeHtml(s.shell)}</div>
              <div class="shell-hook-file">${shortFile}</div>
            </div>
          </div>
          ${s.installed
            ? '<span class="version-badge badge-installed">Installed</span>'
            : `<button class="btn btn-small btn-primary" data-action="install-hook" data-shell="${escapeHtml(s.shell)}">Install Hook</button>`
          }
        </div>
      `;
    }).join('');
  } catch (e) {
    container.innerHTML = `<p style="color:var(--slate-500)">Could not detect shells: ${escapeHtml(e)}</p>`;
  }
}

async function doInstallHook(shell) {
  try {
    const result = await invoke('install_shell_hook', { shell });
    showToast(`${result}. Restart your terminal to activate.`, 'success');
    refreshShellHookStatus();
  } catch (e) {
    showToast(e, 'error');
  }
}

async function showShellHook() {
  const shell = document.getElementById('shell-select').value;
  const preview = document.getElementById('shell-hook-preview');
  try {
    const hook = await invoke('get_shell_hook', { shell });
    preview.textContent = hook;
    preview.classList.remove('hidden');
  } catch (e) {
    showToast(e, 'error');
  }
}

function openExternal(url) {
  window.__TAURI__.shell.open(url);
}

// ============================================
// Gems Panel
// ============================================

let currentGemsData = [];
let currentGemsVersion = '';

async function installGemFromInput() {
  const input = document.getElementById('gem-install-input');
  const versionInput = document.getElementById('gem-version-input');
  const btn = document.getElementById('gem-install-btn');
  const gemName = input.value.trim().toLowerCase();
  const gemVersion = versionInput.value.trim() || undefined;

  if (!gemName || !currentGemsVersion) return;

  const displayName = gemVersion ? `${gemName} ${gemVersion}` : gemName;
  btn.disabled = true;
  btn.textContent = 'Installing...';

  try {
    const params = { rubyVersion: currentGemsVersion, gemName: gemName };
    if (gemVersion) params.gemVersion = gemVersion;
    const result = await invoke('install_gem', params);
    showToast(`Installed ${displayName}`, 'success');
    input.value = '';
    versionInput.value = '';

    // Optimistic update: add the gem to local data and re-render
    if (!currentGemsData.find(g => g.name === gemName)) {
      currentGemsData.push({ name: gemName, version: gemVersion || 'latest', is_default: false });
      currentGemsData.sort((a, b) => a.name.toLowerCase().localeCompare(b.name.toLowerCase()));
    }
    const panel = document.getElementById(`panel-${currentGemsVersion}`);
    if (panel) {
      renderInlineGems(panel, currentGemsVersion, currentGemsData);
    }
  } catch (e) {
    showToast(`Failed to install ${displayName}: ${e}`, 'error');
  } finally {
    btn.disabled = false;
    btn.textContent = 'Install';
  }
}

async function removeGem(gemName) {
  const confirmed = await showConfirm(
    'Uninstall Gem',
    `Remove ${gemName} from Ruby ${currentGemsVersion}?`
  );
  if (!confirmed) return;

  try {
    await invoke('uninstall_gem', {
      rubyVersion: currentGemsVersion,
      gemName: gemName
    });
    showToast(`Removed ${gemName}`, 'success');

    // Immediately remove from local data so UI updates even if re-fetch returns stale data
    currentGemsData = currentGemsData.filter(g => g.name !== gemName);

    // Re-render the panel with updated data
    const panel = document.getElementById(`panel-${currentGemsVersion}`);
    if (panel) {
      renderInlineGems(panel, currentGemsVersion, currentGemsData);
    }
  } catch (e) {
    showToast(`Failed to remove ${gemName}: ${e}`, 'error');
  }
}

function filterGems() {
  const search = document.getElementById('gems-search')?.value.toLowerCase() || '';
  const showDefault = document.getElementById('gems-show-default')?.checked ?? true;
  const showUser = document.getElementById('gems-show-user')?.checked ?? true;

  const filtered = currentGemsData.filter(gem => {
    const matchesSearch = gem.name.toLowerCase().includes(search) || gem.version.includes(search);
    const matchesType = (gem.is_default && showDefault) || (!gem.is_default && showUser);
    return matchesSearch && matchesType;
  });

  // Re-render the gems list inside the current panel
  const gemsList = document.getElementById('gems-list');
  if (!gemsList) return;

  if (filtered.length === 0) {
    gemsList.innerHTML = '<div class="empty-state" style="grid-column:1/-1;padding:24px"><p>No gems match your filter</p></div>';
    return;
  }

  gemsList.innerHTML = filtered.map(gem => `
    <div class="gem-item">
      <div>
        <span class="gem-name">${escapeHtml(gem.name)}</span>
        <span class="${gem.is_default ? 'gem-badge-default' : 'gem-badge-user'}">${gem.is_default ? 'default' : 'user'}</span>
      </div>
      <div class="gem-right">
        <span class="gem-version">${escapeHtml(gem.version)}</span>
        ${!gem.is_default ? `<button class="gem-remove-btn" data-action="remove-gem" data-gem="${escapeHtml(gem.name)}" title="Uninstall">&#10005;</button>` : ''}
      </div>
    </div>
  `).join('');
}

// ============================================
// Info Popups
// ============================================

const INFO_CONTENT = {
  dashboard: {
    title: 'Dashboard',
    body: `<p>The Dashboard shows an overview of your Ruby environment: how many versions you have installed, which one is currently active, and how many more are available to download.</p>
<p>Each installed version has <strong>Gems</strong> and <strong>Projects</strong> buttons that expand inline panels where you can browse, install, or remove gems, and see which projects use that version.</p>
<p>The <strong>Use</strong> dropdown lets you set a version as your global default (used in all terminals) or pin it to a specific project folder by writing a <code>.ruby-version</code> file.</p>`
  },
  install: {
    title: 'Install Ruby',
    body: `<p>Rubynaut installs pre-built Ruby binaries from the <strong>ruby-builder</strong> project &mdash; the same source that GitHub Actions uses. Installation takes seconds instead of the 5&ndash;15 minutes needed to compile from source.</p>
<p>Supported engines include <strong>CRuby</strong> (the standard Ruby), <strong>JRuby</strong> (Ruby on the JVM), and <strong>TruffleRuby</strong> (high-performance Ruby from Oracle).</p>
<p>Every download is verified with a <strong>SHA256 checksum</strong> to ensure the file hasn't been corrupted or tampered with.</p>`
  },
  projects: {
    title: 'Project Tracking',
    body: `<p>Open a project folder and Rubynaut will automatically detect which Ruby version it needs by reading one of these files:</p>
<p>&bull; <strong>.ruby-version</strong> &mdash; the standard version file used by rbenv and others<br>
&bull; <strong>.tool-versions</strong> &mdash; the format used by asdf and mise<br>
&bull; <strong>Gemfile</strong> &mdash; the <code>ruby "3.3.6"</code> constraint in your Gemfile</p>
<p>If the required version isn't installed, you can install it with one click. Tracked projects appear in a list so you can see the status of all your Ruby projects at a glance.</p>`
  },
  doctor: {
    title: 'Environment Doctor',
    body: `<p>Doctor runs <strong>9 diagnostic checks</strong> on your system to make sure everything Ruby needs is in place:</p>
<p>&bull; Rubies directory exists<br>
&bull; At least one Ruby version installed<br>
&bull; Shell hook is set up<br>
&bull; <code>ruby</code> resolves correctly in PATH<br>
&bull; No conflicting managers (rbenv, rvm, etc.)<br>
&bull; C compiler available (for native gem extensions)<br>
&bull; Shared libraries: libyaml, OpenSSL, libffi, GMP</p>
<p>Items marked with an error have a <strong>Fix</strong> button that runs the appropriate install command for your platform (e.g. <code>brew install libyaml</code>).</p>`
  },
  settings: {
    title: 'Settings',
    body: `<p>Configure your <strong>shell integration</strong> so Ruby versions switch automatically when you <code>cd</code> into a project with a <code>.ruby-version</code> file.</p>
<p>Rubynaut supports <strong>bash</strong>, <strong>zsh</strong>, <strong>fish</strong>, and <strong>PowerShell</strong>. Click "Install Hook" to add the integration to your shell config, or use "Show Hook" to copy the code manually.</p>
<p>The <strong>About</strong> section has links to the GitHub repository, issue tracker, and contribution guide.</p>`
  }
};

function showInfoPopup(infoKey) {
  const info = INFO_CONTENT[infoKey];
  if (!info) return;

  // Remove any existing popup
  closeInfoPopup();

  const popup = document.createElement('div');
  popup.className = 'info-popup-overlay';
  popup.innerHTML = `
    <div class="info-popup">
      <div class="info-popup-header">
        <span class="info-popup-icon">i</span>
        <h3>${escapeHtml(info.title)}</h3>
        <button class="info-popup-close" data-action="close-info">&times;</button>
      </div>
      <div class="info-popup-body">${info.body}</div>
    </div>
  `;
  document.body.appendChild(popup);

  // Close on overlay click
  popup.addEventListener('click', (e) => {
    if (e.target === popup) closeInfoPopup();
  });
}

function closeInfoPopup() {
  const existing = document.querySelector('.info-popup-overlay');
  if (existing) existing.remove();
}

function triggerWizard() {
  // Reset wizard flag and show it
  invoke('set_wizard_completed').catch(() => {});
  showWizard();
}

function dismissWizard() {
  invoke('set_wizard_completed').catch(() => {});
  document.getElementById('wizard-overlay').classList.add('hidden');
  refreshDashboard();
}

// ============================================
// Getting Started Wizard
// ============================================

let wizardStep = 1;
let wizardVersion = '';
const WIZARD_TOTAL_STEPS = 5;

async function checkWizard() {
  try {
    const [config, installed] = await Promise.all([
      invoke('get_config'),
      invoke('get_installed_rubies'),
    ]);
    if (!config.wizard_completed && installed.length === 0) {
      showWizard();
    }
  } catch (e) {
    // Config or rubies call failed — skip wizard
  }
}

function showWizard() {
  const overlay = document.getElementById('wizard-overlay');
  overlay.classList.remove('hidden');
  wizardStep = 1;
  renderWizardStep(1);
}

function renderWizardStep(step) {
  wizardStep = step;
  // Update dots
  const dotsContainer = document.getElementById('wizard-dots');
  dotsContainer.innerHTML = Array.from({ length: WIZARD_TOTAL_STEPS }, (_, i) => {
    const cls = i + 1 === step ? 'active' : (i + 1 < step ? 'done' : '');
    return `<div class="wizard-dot ${cls}"></div>`;
  }).join('');

  // Show/hide steps
  document.querySelectorAll('.wizard-step').forEach(el => {
    el.classList.toggle('active', parseInt(el.dataset.step) === step);
  });

  // Per-step setup
  if (step === 2) {
    setupWizardInstallStep();
  } else if (step === 3) {
    document.getElementById('wizard-global-status').textContent = `Ruby ${wizardVersion} is now your global default`;
  } else if (step === 4) {
    setupWizardShellStep();
  } else if (step === 5) {
    renderWhatsNextPanel(document.getElementById('wizard-whats-next'));
  }
}

async function setupWizardInstallStep() {
  const statusEl = document.getElementById('wizard-install-version');
  const installBtn = document.getElementById('wizard-install-btn');
  statusEl.textContent = 'Detecting latest version...';
  try {
    wizardVersion = await invoke('get_latest_stable_version');
  } catch (e) {
    wizardVersion = '4.0.2';
  }

  // Check if already installed
  try {
    const installed = await invoke('get_installed_rubies');
    const alreadyInstalled = installed.some(r => r.version === wizardVersion);
    if (alreadyInstalled) {
      statusEl.textContent = `Ruby ${wizardVersion} is already installed`;
      statusEl.classList.add('success');
      // Set as global and skip to step 3
      await invoke('set_global_version', { version: wizardVersion });
      installBtn.textContent = 'Already Installed';
      installBtn.disabled = true;
      // Auto-advance after a brief pause so the user sees the message
      setTimeout(() => renderWizardStep(3), 800);
      return;
    }
  } catch (e) {
    // Could not check — proceed with install button
  }

  statusEl.textContent = `Ruby ${wizardVersion} (latest stable)`;
  statusEl.classList.remove('success');
  installBtn.textContent = 'Install';
  installBtn.disabled = false;
}

async function wizardInstallRuby() {
  const btn = document.getElementById('wizard-install-btn');
  btn.disabled = true;
  btn.textContent = 'Installing...';

  const progress = document.getElementById('wizard-progress');
  progress.classList.remove('hidden');

  // Listen for progress in wizard
  const unlisten = await listen('install-progress', (event) => {
    const { percent, message } = event.payload;
    document.getElementById('wizard-progress-fill').style.width = `${percent}%`;
    document.getElementById('wizard-progress-msg').textContent = message;
  });

  try {
    await invoke('install_ruby', { version: wizardVersion });
    await invoke('set_global_version', { version: wizardVersion });
    unlisten();
    progress.classList.add('hidden');

    // Auto-advance to step 3
    renderWizardStep(3);
  } catch (e) {
    unlisten();
    btn.disabled = false;
    btn.textContent = 'Retry';
    document.getElementById('wizard-progress-msg').textContent = `Failed: ${e}`;
  }
}

async function setupWizardShellStep() {
  const statusEl = document.getElementById('wizard-shell-status');
  try {
    const hooks = await invoke('check_shell_hook');
    const installed = hooks.find(h => h.installed);
    if (installed) {
      statusEl.textContent = `Shell hook already installed for ${installed.shell}`;
      statusEl.classList.add('success');
      document.getElementById('wizard-hook-btn').textContent = 'Already Installed';
      document.getElementById('wizard-hook-btn').disabled = true;
    } else {
      const shell = hooks.length > 0 ? hooks[0].shell : 'zsh';
      statusEl.textContent = `Detected shell: ${shell}`;
      statusEl.dataset.shell = shell;
    }
  } catch (e) {
    statusEl.textContent = 'Could not detect shell';
  }
}

async function wizardInstallHook() {
  const statusEl = document.getElementById('wizard-shell-status');
  const shell = statusEl.dataset.shell || 'zsh';
  const btn = document.getElementById('wizard-hook-btn');
  btn.disabled = true;
  btn.textContent = 'Installing...';

  try {
    await invoke('install_shell_hook', { shell });
    statusEl.textContent = `Hook installed for ${shell}`;
    statusEl.classList.add('success');
    btn.textContent = 'Installed';
  } catch (e) {
    btn.disabled = false;
    btn.textContent = 'Retry';
    showToast(`Hook install failed: ${e}`, 'error');
  }
}

async function wizardComplete() {
  // Install Rails if checked
  const railsCheckbox = document.getElementById('wizard-install-rails');
  if (railsCheckbox && railsCheckbox.checked && wizardVersion) {
    showToast('Installing Rails... this may take a minute', 'info');
    try {
      await invoke('install_gem', { rubyVersion: wizardVersion, gemName: 'rails' });
      showToast('Rails installed!', 'success');
    } catch (e) {
      showToast(`Rails install failed: ${e}`, 'error');
    }
  }

  // Mark wizard as completed
  await invoke('set_wizard_completed');
  document.getElementById('wizard-overlay').classList.add('hidden');
  refreshDashboard();
}

// ============================================
// Init
// ============================================

async function init() {
  // Listen for install progress events from the Rust backend
  listen('install-progress', (event) => {
    const { stage, percent, message } = event.payload;
    document.getElementById('install-progress-stage').textContent =
      { download: 'Downloading', extract: 'Extracting', verify: 'Verifying', done: 'Complete' }[stage] || stage;
    document.getElementById('install-progress-percent').textContent = `${percent}%`;
    document.getElementById('install-progress-fill').style.width = `${percent}%`;
    document.getElementById('install-progress-message').textContent = message;
  });

  // Global event delegation — handles all data-action clicks
  document.addEventListener('click', (e) => {
    const target = e.target.closest('[data-action]');
    if (!target) return;

    const action = target.dataset.action;
    switch (action) {
      case 'switch-tab':
        switchTab(target.dataset.tab);
        break;
      case 'open-project':
        openProject();
        break;
      case 'run-doctor':
        runDoctor();
        break;
      case 'show-shell-hook':
        showShellHook();
        break;
      case 'open-external':
        e.preventDefault();
        openExternal(target.dataset.url);
        break;
      case 'toggle-panel':
        togglePanel(target.dataset.version, target.dataset.panel);
        break;
      case 'toggle-version-menu':
        toggleVersionMenu(target.dataset.version);
        break;
      case 'set-global':
        setGlobal(target.dataset.version);
        break;
      case 'set-local':
        promptSetLocal(target.dataset.version);
        break;
      case 'uninstall-ruby':
        uninstallRuby(target.dataset.version);
        break;
      case 'install-gem':
        installGemFromInput();
        break;
      case 'remove-gem':
        removeGem(target.dataset.gem);
        break;
      case 'install-ruby':
        installRuby(target.dataset.version);
        break;
      case 'install-for-project':
        installForProject(target.dataset.version, target.dataset.path);
        break;
      case 'set-global-from-project':
        setGlobalFromProject(target.dataset.version);
        break;
      case 'run-bundle':
        runBundleInstall(target.dataset.version, target.dataset.path, parseInt(target.dataset.idx));
        break;
      case 'toggle-project-gems':
        toggleProjectGems(target.dataset.path, parseInt(target.dataset.idx));
        break;
      case 'remove-tracked-project':
        removeTrackedProject(target.dataset.path);
        break;
      case 'run-doctor-fix':
        runDoctorFix(target.dataset.command, parseInt(target.dataset.idx));
        break;
      case 'install-hook':
        doInstallHook(target.dataset.shell);
        break;
      case 'copy-hello-ruby':
        copyHelloRuby();
        break;
      case 'wizard-next':
        renderWizardStep(wizardStep + 1);
        break;
      case 'wizard-back':
        renderWizardStep(wizardStep - 1);
        break;
      case 'wizard-install-ruby':
        wizardInstallRuby();
        break;
      case 'wizard-install-hook':
        wizardInstallHook();
        break;
      case 'wizard-skip-shell':
        renderWizardStep(wizardStep + 1);
        break;
      case 'wizard-complete':
        wizardComplete();
        break;
      case 'trigger-wizard':
        triggerWizard();
        break;
      case 'wizard-dismiss':
        dismissWizard();
        break;
      case 'filter-engine':
        filterEngine(target.dataset.engine);
        break;
      case 'show-info':
        showInfoPopup(target.dataset.info);
        break;
      case 'close-info':
        closeInfoPopup();
        break;
    }
  });

  // Event delegation for input events (gem filter, version search, Enter key)
  document.addEventListener('input', (e) => {
    if (e.target.id === 'gems-search' || e.target.id === 'gems-show-default' || e.target.id === 'gems-show-user') {
      filterGems();
    }
    if (e.target.id === 'install-search') {
      renderAvailableVersions();
    }
  });
  document.addEventListener('change', (e) => {
    if (e.target.id === 'gems-show-default' || e.target.id === 'gems-show-user') {
      filterGems();
    }
  });
  document.addEventListener('keydown', (e) => {
    if (e.key === 'Enter' && (e.target.id === 'gem-install-input' || e.target.id === 'gem-version-input')) {
      installGemFromInput();
    }
  });

  await detectPlatform();
  await refreshDashboard();
  await checkWizard();
}

// Wait for both DOM and Tauri to be ready
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', init);
} else {
  init();
}
