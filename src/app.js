// ─── Tauri API ───
const { invoke } = window.__TAURI__.core;
const { open, save } = window.__TAURI__.dialog;
const { listen } = window.__TAURI__.event;

// ─── State ───
let groups = [];
let files = [];
let selectedGroup = 0;
let showConflictsOnly = false;
let searchQuery = '';
let showPasswords = false;
let editingCell = null;

const FIELDS = ['Username', 'Password', 'URL', 'Notes', 'Created', 'Updated'];
const FIELD_ICONS = {
  Username: 'ph-user',
  Password: 'ph-key',
  URL: 'ph-globe',
  Notes: 'ph-note-pencil',
  Created: 'ph-calendar',
  Updated: 'ph-arrows-clockwise',
};

// ─── DOM ───
const $ = (s) => document.querySelector(s);
const btnImport = $('#btn-import');
const btnImportEmpty = $('#btn-import-empty');
const btnExport = $('#btn-export');
const conflictsOnly = $('#conflicts-only');
const searchInput = $('#search');
const statusMsg = $('#status-msg');
const fileBadge = $('#file-badge');
const fileChips = $('#file-chips');
const chipList = $('#chip-list');
const emptyState = $('#empty-state');
const mainLayout = $('#main-layout');
const entryList = $('#entry-list');
const comparePanel = $('#compare-panel');

// ─── File Import via Tauri Dialog ───
async function pickFiles() {
  try {
    const selected = await open({
      multiple: true,
      filters: [{ name: 'CSV', extensions: ['csv'] }],
    });
    if (selected) {
      const paths = Array.isArray(selected) ? selected : [selected];
      if (paths.length > 0) await importFiles(paths);
    }
  } catch (err) {
    statusMsg.textContent = 'Error: ' + err;
  }
}

async function importFiles(paths) {
  try {
    await invoke('import_files', { paths });
    files = await invoke('get_files');
    groups = await invoke('get_groups');
    selectedGroup = 0;
    const count = files.reduce((s, f) => s + f.entries.length, 0);
    const names = files.map(f => f.name).join(', ');
    statusMsg.textContent = `Loaded ${names} (${count} entries)`;
    updateView();
  } catch (err) {
    statusMsg.textContent = 'Error: ' + err;
  }
}

async function handleExport() {
  if (groups.length === 0) return;
  try {
    const path = await save({
      title: 'Save merged CSV',
      filters: [{ name: 'CSV', extensions: ['csv'] }],
    });
    if (path) {
      await invoke('export_csv', { path });
      statusMsg.textContent = 'Exported successfully!';
    }
  } catch (err) {
    statusMsg.textContent = 'Export error: ' + err;
  }
}

// ─── Events ───
btnImport.addEventListener('click', pickFiles);
btnImportEmpty.addEventListener('click', pickFiles);
btnExport.addEventListener('click', handleExport);
conflictsOnly.addEventListener('change', () => {
  showConflictsOnly = conflictsOnly.checked;
  renderEntryList();
});
searchInput.addEventListener('input', (e) => {
  searchQuery = e.target.value;
  renderEntryList();
});

// Drag and drop via Tauri events
listen('tauri://drag-drop', async (event) => {
  const paths = event.payload.paths.filter(p => p.endsWith('.csv'));
  if (paths.length > 0) {
    await importFiles(paths);
  }
});

// Resize handle
const resizeHandle = $('#resize-handle');
const sidebar = $('#sidebar');
let isResizing = false;
resizeHandle.addEventListener('mousedown', () => {
  isResizing = true;
  document.addEventListener('mousemove', onResize);
  document.addEventListener('mouseup', () => {
    isResizing = false;
    document.removeEventListener('mousemove', onResize);
  }, { once: true });
});
function onResize(e) {
  if (!isResizing) return;
  const w = Math.min(350, Math.max(200, e.clientX));
  sidebar.style.width = w + 'px';
}

// ─── View Updates ───
function updateView() {
  const hasFiles = files.length > 0;
  emptyState.style.display = hasFiles ? 'none' : 'flex';
  mainLayout.style.display = hasFiles ? 'flex' : 'none';
  btnExport.disabled = !hasFiles;
  fileBadge.style.display = hasFiles ? '' : 'none';
  fileBadge.textContent = `${files.length} file(s)`;
  fileChips.style.display = hasFiles ? '' : 'none';
  searchInput.style.display = hasFiles ? '' : 'none';
  renderChips();
  renderEntryList();
  renderComparison();
}

function renderChips() {
  chipList.innerHTML = '';
  files.forEach((file, i) => {
    const chip = document.createElement('div');
    chip.className = 'chip';
    chip.innerHTML = `
      <span class="chip-name">${esc(file.name)}</span>
      <span class="chip-count">(${file.entries.length})</span>
      <span class="chip-close" data-idx="${i}">×</span>
    `;
    chip.querySelector('.chip-close').addEventListener('click', async () => {
      await invoke('remove_file', { idx: i });
      files = await invoke('get_files');
      groups = await invoke('get_groups');
      if (selectedGroup >= groups.length) selectedGroup = Math.max(0, groups.length - 1);
      statusMsg.textContent = `Removed file`;
      updateView();
    });
    chipList.appendChild(chip);
  });
}

function renderEntryList() {
  entryList.innerHTML = '';
  const query = searchQuery.toLowerCase();
  const filtered = groups
    .map((g, i) => ({ g, i }))
    .filter(({ g }) => {
      if (showConflictsOnly && !g.has_conflicts) return false;
      if (query && !g.title.toLowerCase().includes(query) && !g.username.toLowerCase().includes(query)) return false;
      return true;
    });

  if (filtered.length === 0) {
    entryList.innerHTML = '<p style="padding:20px;color:var(--gray);font-size:12px;text-align:center">No entries found</p>';
    return;
  }

  filtered.forEach(({ g, i }) => {
    const conflicts = g.has_conflicts;
    const resolved = g.resolved_source !== null;
    const isSelected = i === selectedGroup;

    const card = document.createElement('div');
    card.className = 'entry-card' + (isSelected ? ' selected' : '');

    let badgeClass = 'badge-synced';
    let badgeText = 'Synced';
    if (resolved) { badgeClass = 'badge-resolved'; badgeText = 'Resolved'; }
    else if (conflicts) { badgeClass = 'badge-conflicts'; badgeText = 'Conflicts'; }

    let info = '';
    if (conflicts) {
      const count = g.conflict_count || 0;
      info = `<div class="entry-card-conflicts">⚠ ${count} differ${count !== 1 ? 's' : ''}</div>`;
    } else if (resolved) {
      info = `<div class="entry-card-resolved">✓ Resolved</div>`;
    }

    card.innerHTML = `
      <div class="entry-card-header">
        <span class="entry-card-title">${esc(g.title)}</span>
        <span class="badge ${badgeClass}">${badgeText}</span>
      </div>
      ${g.username ? `<div class="entry-card-user">${esc(g.username)}</div>` : ''}
      ${info}
    `;

    card.addEventListener('click', () => {
      selectedGroup = i;
      editingCell = null;
      renderEntryList();
      renderComparison();
    });

    entryList.appendChild(card);
  });
}

function renderComparison() {
  if (groups.length === 0) {
    comparePanel.innerHTML = '<p style="text-align:center;color:var(--gray);padding:80px">No entries</p>';
    return;
  }

  const g = groups[selectedGroup];
  if (!g) return;

  const conflicts = g.has_conflicts;
  const resolved = g.resolved_source !== null;
  const numFiles = g.entries.length;

  const seen = {};
  const fileLabels = g.entries.map(([fi]) => {
    const name = files[fi]?.name || `File ${fi}`;
    const nth = (seen[fi] = (seen[fi] || 0) + 1);
    const total = g.entries.filter(([i]) => i === fi).length;
    return total > 1 ? `${name} (${nth})` : name;
  });

  const standardKeys = new Set([
    'title','username','password','url','notes','created_at','updated_at',
    'created','updated','login_username','login_password','login_uri','login_name',
    'name','website','uri','note','extra','grouping','folder','favorite','favourite',
    'fav','type','fields','reprompt','totp','timecreated','timepasswordchanged',
    'timelastused','last_modified','group','email','createtime','modifytime','vault',
  ]);
  const firstEntry = g.entries[0]?.[1];
  const extras = firstEntry ? Object.keys(firstEntry.raw || {}).filter(k => !standardKeys.has(k)) : [];

  let html = '';

  // Header
  html += `<div class="compare-header">`;
  html += `<h2>${esc(g.title)}</h2>`;
  if (g.username) html += `<span class="username">(${esc(g.username)})</span>`;
  if (conflicts) {
    html += `<span class="badge badge-conflicts">⚠ Conflicts</span>`;
  } else if (numFiles > 1) {
    html += `<span class="badge badge-synced">✓ Synced</span>`;
  }
  if (numFiles > 1) {
    html += `<div class="right"><button class="btn" id="toggle-pw">
      <i class="ph ${showPasswords ? 'ph-eye-slash' : 'ph-eye'}"></i> Passwords
    </button></div>`;
  }
  html += `</div>`;

  // Grid
  html += `<div class="compare-grid">`;
  g.entries.forEach(([fileIdx, entry], globalIdx) => {
    const isResolved = resolved && globalIdx === g.resolved_source;
    const isResolvedBox = isResolved && conflicts;
    let boxClass = 'entry-box';
    if (isResolvedBox) boxClass += ' resolved';
    else if (conflicts) boxClass += ' conflict';
    else if (resolved) boxClass += ' has-resolved';
    else boxClass += ' normal';

    html += `<div class="${boxClass}">`;
    html += `<div class="entry-box-header">`;
    if (isResolvedBox) html += `<i class="ph ph-check check-icon"></i> `;
    html += `${esc(fileLabels[globalIdx])}</div>`;

    FIELDS.forEach((field) => {
      const val = entry[field.toLowerCase().replace(' ', '_')] || '';
      const isEditing = editingCell
        && editingCell.groupIdx === selectedGroup
        && editingCell.fileIdx === globalIdx
        && editingCell.field === field;
      const isPassword = field === 'Password';
      const editable = ['Username', 'Password', 'URL', 'Notes'].includes(field);

      let valueHtml = '';
      if (isEditing) {
        valueHtml = `
          <div class="field-edit">
            <input type="text" class="edit-input" data-group="${selectedGroup}" data-file="${globalIdx}" data-field="${field}" value="${esc(val)}">
            <button class="btn btn-green edit-done" data-group="${selectedGroup}" data-file="${globalIdx}" data-field="${field}">✓</button>
          </div>`;
      } else {
        let displayVal = val;
        let valClass = 'field-value';
        if (isPassword && !showPasswords && val) displayVal = '••••••••';
        else if (!val) { displayVal = '—'; valClass += ' empty'; }
        else if (isResolvedBox) valClass += ' resolved-val';
        else if (resolved) valClass += ' dimmed';

        let editBtn = '';
        if (editable && val) {
          editBtn = `<i class="ph ph-pencil-simple edit-icon" data-group="${selectedGroup}" data-file="${globalIdx}" data-field="${field}"></i>`;
        }
        valueHtml = `${editBtn}<span class="${valClass}">${esc(displayVal)}</span>`;
      }

      html += `<div class="field-row">
        <span class="field-label"><i class="ph ${FIELD_ICONS[field]}"></i> ${field}</span>
        ${valueHtml}
      </div>`;
    });

    html += `<button class="keep-btn" data-group="${selectedGroup}" data-file="${fileIdx}">✓ Keep</button>`;
    html += `</div>`;
  });
  html += `</div>`;

  // Extra fields
  if (extras.length > 0) {
    html += `<div class="extra-fields-toggle" id="extra-toggle">
      <i class="ph ph-paperclip"></i> Extra fields (${extras.length})
      <span style="margin-left:auto">▶</span>
    </div>`;
    html += `<div class="extra-fields-content" id="extra-content" style="display:none">`;
    extras.forEach(key => {
      html += `<div class="extra-field-row">
        <span class="extra-field-name">${esc(key)}</span>`;
      g.entries.forEach(([, e]) => {
        const v = e.raw?.[key] || '—';
        html += `<span style="flex:1;text-align:right">${esc(v)}</span>`;
      });
      html += `</div>`;
    });
    html += `</div>`;
  }

  // Resolved footer
  if (resolved && numFiles > 1) {
    const resolvedIdx = g.entries.findIndex(([i]) => i === g.resolved_source);
    const resolvedLabel = resolvedIdx >= 0 ? fileLabels[resolvedIdx] : '';
    html += `<div class="resolved-footer">
      <span>Resolved from</span>
      <span class="resolved-source">${esc(resolvedLabel)}</span>
      <button class="clear-btn" id="clear-resolve">Clear</button>
    </div>`;
  }

  comparePanel.innerHTML = html;
  bindCompareEvents();
}

function bindCompareEvents() {
  const togglePw = $('#toggle-pw');
  if (togglePw) togglePw.addEventListener('click', () => { showPasswords = !showPasswords; renderComparison(); });

  document.querySelectorAll('.keep-btn').forEach(btn => {
    btn.addEventListener('click', async () => {
      await invoke('resolve_group', { groupIdx: +btn.dataset.group, entryFileIdx: +btn.dataset.file });
      groups = await invoke('get_groups');
      renderEntryList();
      renderComparison();
    });
  });

  document.querySelectorAll('.edit-icon').forEach(icon => {
    icon.addEventListener('click', () => {
      editingCell = { groupIdx: +icon.dataset.group, fileIdx: +icon.dataset.file, field: icon.dataset.field };
      renderComparison();
      const input = comparePanel.querySelector('.edit-input');
      if (input) { input.focus(); input.select(); }
    });
  });

  document.querySelectorAll('.edit-done').forEach(btn => {
    btn.addEventListener('click', () => commitEdit(+btn.dataset.group, +btn.dataset.file, btn.dataset.field));
  });

  document.querySelectorAll('.edit-input').forEach(input => {
    input.addEventListener('keydown', (e) => {
      if (e.key === 'Enter') commitEdit(+input.dataset.group, +input.dataset.file, input.dataset.field);
      if (e.key === 'Escape') { editingCell = null; renderComparison(); }
    });
  });

  const extraToggle = $('#extra-toggle');
  const extraContent = $('#extra-content');
  if (extraToggle && extraContent) {
    extraToggle.addEventListener('click', () => {
      const visible = extraContent.style.display !== 'none';
      extraContent.style.display = visible ? 'none' : '';
      extraToggle.querySelector('span').textContent = visible ? '▶' : '▼';
    });
  }

  const clearBtn = $('#clear-resolve');
  if (clearBtn) {
    clearBtn.addEventListener('click', async () => {
      await invoke('clear_resolve', { groupIdx: selectedGroup });
      groups = await invoke('get_groups');
      renderEntryList();
      renderComparison();
    });
  }
}

async function commitEdit(groupIdx, fileIdx, field) {
  const input = comparePanel.querySelector(`.edit-input[data-group="${groupIdx}"][data-file="${fileIdx}"][data-field="${field}"]`);
  if (!input) return;
  await invoke('edit_field', { groupIdx, entryFileIdx: fileIdx, field, value: input.value });
  groups = await invoke('get_groups');
  editingCell = null;
  renderEntryList();
  renderComparison();
}

function esc(s) {
  if (!s) return '';
  const d = document.createElement('div');
  d.textContent = s;
  return d.innerHTML;
}
