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
let showAllFields = true;

const FIELDS = ['Username', 'Password', 'URL', 'Notes', 'Created', 'Updated'];
const FIELD_ICONS = {
  Username: 'ph-user', Password: 'ph-key', URL: 'ph-globe',
  Notes: 'ph-note-pencil', Created: 'ph-calendar', Updated: 'ph-arrows-clockwise',
};
const FILE_COLORS = ['chip-0', 'chip-1', 'chip-2', 'chip-3'];

// ─── DOM ───
const $ = (s) => document.querySelector(s);
const statusMsg = $('#status-msg');

// ─── Toast ───
function toast(msg, type = 'info') {
  const el = document.createElement('div');
  el.className = 'toast' + (type === 'error' ? ' error' : type === 'success' ? ' success' : '');
  el.textContent = msg;
  $('#toast-container').appendChild(el);
  setTimeout(() => { el.classList.add('fade-out'); }, 2500);
  setTimeout(() => el.remove(), 2800);
}

// ─── Helpers ───
function esc(s) {
  if (!s) return '';
  const d = document.createElement('div');
  d.textContent = s;
  return d.innerHTML;
}

function getConflictingFields(group) {
  if (!group.has_conflicts || group.entries.length < 2) return new Set();
  const base = group.entries[0][1];
  const conflicting = new Set();
  FIELDS.forEach(field => {
    const key = field.toLowerCase().replace(' ', '_');
    const baseVal = base[key] || '';
    for (let i = 1; i < group.entries.length; i++) {
      if ((group.entries[i][1][key] || '') !== baseVal) {
        conflicting.add(field);
        break;
      }
    }
  });
  return conflicting;
}

// ─── File Import ───
async function pickFiles() {
  try {
    const selected = await open({ multiple: true, filters: [{ name: 'CSV', extensions: ['csv'] }] });
    if (selected) {
      const paths = Array.isArray(selected) ? selected : [selected];
      if (paths.length > 0) await importFiles(paths);
    }
  } catch (err) { toast('Error: ' + err, 'error'); }
}

async function importFiles(paths) {
  try {
    await invoke('import_files', { paths });
    files = await invoke('get_files');
    groups = await invoke('get_groups');
    selectedGroup = 0;
    const count = files.reduce((s, f) => s + f.entries.length, 0);
    toast(`Loaded ${files.length} file(s) — ${count} entries`, 'success');
    updateView();
  } catch (err) { toast('Error: ' + err, 'error'); }
}

async function handleExport() {
  if (groups.length === 0) return;
  try {
    const path = await save({ title: 'Save merged CSV', filters: [{ name: 'CSV', extensions: ['csv'] }] });
    if (path) {
      await invoke('export_csv', { path });
      toast('Exported successfully!', 'success');
    }
  } catch (err) { toast('Export error: ' + err, 'error'); }
}

// ─── Event Bindings ───
$('#btn-import').addEventListener('click', pickFiles);
$('#btn-import-empty').addEventListener('click', pickFiles);
$('#btn-export').addEventListener('click', handleExport);
$('#conflicts-only').addEventListener('change', (e) => {
  showConflictsOnly = e.target.checked;
  renderEntryList();
});
$('#search').addEventListener('input', (e) => {
  searchQuery = e.target.value;
  renderEntryList();
});

listen('tauri://drag-drop', async (event) => {
  const paths = event.payload.paths.filter(p => p.endsWith('.csv'));
  if (paths.length > 0) await importFiles(paths);
});

// Drop zone visual feedback
const dropZone = $('#drop-zone');
let dragCounter = 0;
document.addEventListener('dragenter', (e) => { e.preventDefault(); dragCounter++; if (dropZone) dropZone.classList.add('drag-over'); });
document.addEventListener('dragleave', (e) => { e.preventDefault(); dragCounter--; if (dragCounter <= 0) { dragCounter = 0; if (dropZone) dropZone.classList.remove('drag-over'); } });
document.addEventListener('dragover', (e) => e.preventDefault());
document.addEventListener('drop', () => { dragCounter = 0; if (dropZone) dropZone.classList.remove('drag-over'); });

// Resize handle
let isResizing = false;
$('#resize-handle').addEventListener('mousedown', () => {
  isResizing = true;
  const onMove = (e) => {
    if (!isResizing) return;
    $('#sidebar').style.width = Math.min(350, Math.max(200, e.clientX)) + 'px';
  };
  const onUp = () => { isResizing = false; document.removeEventListener('mousemove', onMove); document.removeEventListener('mouseup', onUp); };
  document.addEventListener('mousemove', onMove);
  document.addEventListener('mouseup', onUp);
});

// Keyboard shortcuts
document.addEventListener('keydown', (e) => {
  if (e.target.tagName === 'INPUT') return;

  if ((e.ctrlKey || e.metaKey) && e.key === 'o') { e.preventDefault(); pickFiles(); return; }
  if ((e.ctrlKey || e.metaKey) && e.key === 's') { e.preventDefault(); handleExport(); return; }

  if (groups.length === 0) return;
  const filtered = getFilteredIndices();

  if (e.key === 'ArrowDown' || e.key === 'j') {
    e.preventDefault();
    const curIdx = filtered.indexOf(selectedGroup);
    if (curIdx < filtered.length - 1) { selectedGroup = filtered[curIdx + 1]; editingCell = null; renderEntryList(); renderComparison(); }
  }
  if (e.key === 'ArrowUp' || e.key === 'k') {
    e.preventDefault();
    const curIdx = filtered.indexOf(selectedGroup);
    if (curIdx > 0) { selectedGroup = filtered[curIdx - 1]; editingCell = null; renderEntryList(); renderComparison(); }
  }
  if (e.key === 'p') {
    e.preventDefault();
    showPasswords = !showPasswords;
    renderComparison();
  }
  if (e.key === 't') {
    e.preventDefault();
    showAllFields = !showAllFields;
    renderComparison();
  }
});

function getFilteredIndices() {
  const q = searchQuery.toLowerCase();
  return groups.map((g, i) => ({ g, i }))
    .filter(({ g }) => {
      if (showConflictsOnly && !g.has_conflicts) return false;
      if (q && !g.title.toLowerCase().includes(q) && !g.username.toLowerCase().includes(q)) return false;
      return true;
    })
    .map(({ i }) => i);
}

// ─── Rendering ───
function updateView() {
  const has = files.length > 0;
  $('#empty-state').style.display = has ? 'none' : 'flex';
  $('#main-layout').style.display = has ? 'flex' : 'none';
  $('#btn-export').disabled = !has;
  $('#file-badge').style.display = has ? '' : 'none';
  $('#file-badge').textContent = `${files.length} file(s)`;
  $('#file-chips').style.display = has ? '' : 'none';
  $('#search').style.display = has ? '' : 'none';
  renderChips();
  renderEntryList();
  renderComparison();
}

function renderChips() {
  const list = $('#chip-list');
  list.innerHTML = '';
  files.forEach((file, i) => {
    const chip = document.createElement('div');
    chip.className = 'chip ' + FILE_COLORS[i % FILE_COLORS.length];
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
      toast('Removed file', 'info');
      updateView();
    });
    list.appendChild(chip);
  });
}

function renderEntryList() {
  const el = $('#entry-list');
  el.innerHTML = '';
  const filtered = getFilteredIndices();

  if (filtered.length === 0) {
    el.innerHTML = '<p style="padding:20px;color:var(--gray);font-size:12px;text-align:center">No entries</p>';
    return;
  }

  filtered.forEach(i => {
    const g = groups[i];
    const conflicts = g.has_conflicts;
    const resolved = g.resolved_source !== null;
    const isSel = i === selectedGroup;

    const card = document.createElement('div');
    card.className = 'entry-card' + (isSel ? ' selected' : '');

    let badgeCls = 'badge-synced', badgeTxt = 'Synced';
    if (resolved) { badgeCls = 'badge-resolved'; badgeTxt = 'Resolved'; }
    else if (conflicts) { badgeCls = 'badge-conflicts'; badgeTxt = 'Conflicts'; }

    let info = '';
    if (conflicts) {
      const n = g.conflict_count || 0;
      info = `<div class="entry-card-info entry-card-conflicts">⚠ ${n} differ${n !== 1 ? 's' : ''}</div>`;
    } else if (resolved) {
      info = `<div class="entry-card-info entry-card-resolved">✓ Resolved</div>`;
    }

    card.innerHTML = `
      <div class="entry-card-header">
        <span class="entry-card-title">${esc(g.title)}</span>
        <span class="badge ${badgeCls}">${badgeTxt}</span>
      </div>
      ${g.username ? `<div class="entry-card-user">${esc(g.username)}</div>` : ''}
      ${info}
    `;
    card.addEventListener('click', () => { selectedGroup = i; editingCell = null; renderEntryList(); renderComparison(); });
    el.appendChild(card);
  });
}

function renderComparison() {
  const panel = $('#compare-panel');
  if (groups.length === 0) {
    panel.innerHTML = '<div class="empty-compare">No entries</div>';
    return;
  }
  const g = groups[selectedGroup];
  if (!g) { panel.innerHTML = '<div class="empty-compare">No entries</div>'; return; }

  const conflicts = g.has_conflicts;
  const resolved = g.resolved_source !== null;
  const numFiles = g.entries.length;
  const conflictingFields = getConflictingFields(g);

  // File labels with color indicator
  const seen = {};
  const fileLabels = g.entries.map(([fi]) => {
    const name = files[fi]?.name || `File ${fi}`;
    const nth = (seen[fi] = (seen[fi] || 0) + 1);
    const total = g.entries.filter(([i]) => i === fi).length;
    return total > 1 ? `${name} (${nth})` : name;
  });

  // Extra fields
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
  html += `<div class="compare-header"><h2>${esc(g.title)}</h2>`;
  if (g.username) html += `<span class="username">(${esc(g.username)})</span>`;
  if (conflicts) html += `<span class="badge badge-conflicts">⚠ Conflicts</span>`;
  else if (numFiles > 1) html += `<span class="badge badge-synced">✓ Synced</span>`;
  html += `<div class="right">`;
  if (numFiles > 1) {
    html += `<button class="btn" id="toggle-pw" title="Press P"><i class="ph ${showPasswords ? 'ph-eye-slash' : 'ph-eye'}"></i></button>`;
    html += `<button class="btn" id="toggle-fields" title="Press T">${showAllFields ? '◆ All fields' : '◇ Diff only'}</button>`;
  }
  html += `</div></div>`;

  // Grid
  html += `<div class="compare-grid">`;
  g.entries.forEach(([fileIdx, entry], globalIdx) => {
    const isResolved = resolved && globalIdx === g.resolved_source;
    const isSelected = isResolved && conflicts;

    let boxCls = 'entry-box';
    if (isSelected) boxCls += ' resolved-selected';

    const chipColor = FILE_COLORS[fileIdx % FILE_COLORS.length];

    html += `<div class="${boxCls}">`;
    html += `<div class="entry-box-header">`;
    if (isSelected) html += `<i class="ph ph-check check-icon"></i>`;
    html += `<span class="${chipColor}" style="display:inline-block;width:8px;height:8px;border-radius:50%;margin-right:2px"></span>`;
    html += `${esc(fileLabels[globalIdx])}</div>`;

    const fieldsToShow = showAllFields ? FIELDS : FIELDS.filter(f => conflictingFields.has(f));

    fieldsToShow.forEach(field => {
      const val = entry[field.toLowerCase().replace(' ', '_')] || '';
      const isEditing = editingCell && editingCell.groupIdx === selectedGroup && editingCell.fileIdx === globalIdx && editingCell.field === field;
      const isPw = field === 'Password';
      const isConflictField = conflictingFields.has(field);

      let rowCls = 'field-row';
      if (isConflictField && conflicts) rowCls += ' conflict';
      if (isSelected && isConflictField) rowCls += ' resolved';

      let valueHtml = '';
      if (isEditing) {
        valueHtml = `<div class="field-edit">
          <input type="text" class="edit-input" id="edit-input"
            data-group="${selectedGroup}" data-file="${globalIdx}" data-field="${field}"
            value="${esc(val)}">
          <button class="btn edit-done" data-group="${selectedGroup}" data-file="${globalIdx}" data-field="${field}">✓</button>
        </div>`;
      } else {
        let displayVal = val;
        let valCls = 'field-value';
        if (isPw && !showPasswords && val) displayVal = '••••••••';
        else if (!val) { displayVal = '—'; valCls += ' empty'; }
        else if (isSelected) valCls += ' resolved-val';
        else if (resolved) valCls += ' dimmed';

        let editBtn = '';
        if (['Username', 'Password', 'URL', 'Notes'].includes(field) && val) {
          editBtn = `<i class="ph ph-pencil-simple edit-icon" data-group="${selectedGroup}" data-file="${globalIdx}" data-field="${field}"></i>`;
        }
        valueHtml = `${editBtn}<span class="${valCls}">${esc(displayVal)}</span>`;
      }

      html += `<div class="${rowCls}">
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
      html += `<div class="extra-field-row"><span class="extra-field-name">${esc(key)}</span>`;
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
    const ri = g.entries.findIndex(([i]) => i === g.resolved_source);
    const rl = ri >= 0 ? fileLabels[ri] : '';
    html += `<div class="resolved-footer">
      <span>Resolved from</span>
      <span class="resolved-source">${esc(rl)}</span>
      <button class="clear-btn" id="clear-resolve">Clear</button>
    </div>`;
  }

  panel.innerHTML = html;
  bindCompareEvents();
}

function bindCompareEvents() {
  $('#toggle-pw')?.addEventListener('click', () => { showPasswords = !showPasswords; renderComparison(); });
  $('#toggle-fields')?.addEventListener('click', () => { showAllFields = !showAllFields; renderComparison(); });

  document.querySelectorAll('.keep-btn').forEach(btn => {
    btn.addEventListener('click', async () => {
      await invoke('resolve_group', { groupIdx: +btn.dataset.group, entryFileIdx: +btn.dataset.file });
      groups = await invoke('get_groups');
      toast('Resolved!', 'success');
      renderEntryList();
      renderComparison();
    });
  });

  // Edit icons
  document.querySelectorAll('.edit-icon').forEach(icon => {
    icon.addEventListener('click', (e) => {
      e.stopPropagation();
      editingCell = { groupIdx: +icon.dataset.group, fileIdx: +icon.dataset.file, field: icon.dataset.field };
      renderComparison();
      const input = document.getElementById('edit-input');
      if (input) { input.focus(); input.select(); }
    });
  });

  // Done buttons
  document.querySelectorAll('.edit-done').forEach(btn => {
    btn.addEventListener('click', async (e) => {
      e.stopPropagation();
      const gIdx = +btn.dataset.group;
      const fIdx = +btn.dataset.file;
      const field = btn.dataset.field;
      const input = document.getElementById('edit-input');
      if (!input) return;
      await invoke('edit_field', { groupIdx: gIdx, entryFileIdx: fIdx, field, value: input.value });
      groups = await invoke('get_groups');
      editingCell = null;
      renderEntryList();
      renderComparison();
    });
  });

  // Enter/Escape on inputs
  document.querySelectorAll('.edit-input').forEach(input => {
    input.addEventListener('keydown', async (e) => {
      e.stopPropagation();
      if (e.key === 'Enter') {
        const gIdx = +input.dataset.group;
        const fIdx = +input.dataset.file;
        const field = input.dataset.field;
        await invoke('edit_field', { groupIdx: gIdx, entryFileIdx: fIdx, field, value: input.value });
        groups = await invoke('get_groups');
        editingCell = null;
        renderEntryList();
        renderComparison();
      }
      if (e.key === 'Escape') { editingCell = null; renderComparison(); }
    });
  });

  $('#extra-toggle')?.addEventListener('click', () => {
    const c = $('#extra-content');
    const vis = c.style.display !== 'none';
    c.style.display = vis ? 'none' : '';
    $('#extra-toggle span:last-child').textContent = vis ? '▶' : '▼';
  });

  $('#clear-resolve')?.addEventListener('click', async () => {
    await invoke('clear_resolve', { groupIdx: selectedGroup });
    groups = await invoke('get_groups');
    toast('Resolve cleared', 'info');
    renderEntryList();
    renderComparison();
  });
}
