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
  Username: 'user', Password: 'key-round', URL: 'globe',
  Notes: 'file-text', Created: 'calendar', Updated: 'refresh-cw',
};
const FILE_COLORS = ['chip-0', 'chip-1', 'chip-2', 'chip-3'];

// ─── Helpers ───
const $ = (s) => document.querySelector(s);
function esc(s) { if (!s) return ''; const d = document.createElement('div'); d.textContent = s; return d.innerHTML; }
function icon(name, cls = '') { return `<i data-lucide="${name}" class="${cls}"></i>`; }
function refreshIcons() { lucide.createIcons(); }

function toast(msg, type = '') {
  const el = document.createElement('div');
  el.className = 'toast' + (type ? ' ' + type : '');
  el.textContent = msg;
  $('#toast-container').appendChild(el);
  setTimeout(() => { el.classList.add('fade-out'); }, 2500);
  setTimeout(() => el.remove(), 2800);
}

function getConflictingFields(group) {
  if (!group.has_conflicts || group.entries.length < 2) return new Set();
  const base = group.entries[0][1];
  const conflicting = new Set();
  FIELDS.forEach(field => {
    const key = field.toLowerCase().replace(' ', '_');
    const baseVal = base[key] || '';
    for (let i = 1; i < group.entries.length; i++) {
      if ((group.entries[i][1][key] || '') !== baseVal) { conflicting.add(field); break; }
    }
  });
  return conflicting;
}

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

// ─── File Import ───
async function pickFiles() {
  try {
    const selected = await open({ multiple: true, filters: [{ name: 'CSV', extensions: ['csv'] }] });
    if (selected) { const paths = Array.isArray(selected) ? selected : [selected]; if (paths.length > 0) await importFiles(paths); }
  } catch (err) { toast(err, 'error'); }
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
  } catch (err) { toast(err, 'error'); }
}

async function handleExport() {
  if (groups.length === 0) return;
  try {
    const path = await save({ title: 'Save merged CSV', filters: [{ name: 'CSV', extensions: ['csv'] }] });
    if (path) { await invoke('export_csv', { path }); toast('Exported!', 'success'); }
  } catch (err) { toast(err, 'error'); }
}

// ─── Events ───
$('#btn-import').addEventListener('click', pickFiles);
$('#btn-import-empty').addEventListener('click', pickFiles);
$('#btn-export').addEventListener('click', handleExport);
$('#conflicts-only').addEventListener('change', (e) => { showConflictsOnly = e.target.checked; renderEntryList(); });
$('#search').addEventListener('input', (e) => { searchQuery = e.target.value; renderEntryList(); });

listen('tauri://drag-drop', async (event) => {
  const paths = event.payload.paths.filter(p => p.endsWith('.csv'));
  if (paths.length > 0) await importFiles(paths);
});

// Drop zone
const dropZone = $('#drop-zone');
let dragC = 0;
document.addEventListener('dragenter', (e) => { e.preventDefault(); dragC++; dropZone?.classList.add('drag-over'); });
document.addEventListener('dragleave', (e) => { e.preventDefault(); dragC--; if (dragC <= 0) { dragC = 0; dropZone?.classList.remove('drag-over'); } });
document.addEventListener('dragover', (e) => e.preventDefault());
document.addEventListener('drop', () => { dragC = 0; dropZone?.classList.remove('drag-over'); });

// Resize
let resizing = false;
$('#resize-handle').addEventListener('mousedown', () => {
  resizing = true;
  const mv = (e) => { if (resizing) $('#sidebar').style.width = Math.min(350, Math.max(200, e.clientX)) + 'px'; };
  const up = () => { resizing = false; document.removeEventListener('mousemove', mv); document.removeEventListener('mouseup', up); };
  document.addEventListener('mousemove', mv);
  document.addEventListener('mouseup', up);
});

// Keyboard
document.addEventListener('keydown', (e) => {
  if (e.target.tagName === 'INPUT') return;
  if ((e.ctrlKey || e.metaKey) && e.key === 'o') { e.preventDefault(); pickFiles(); }
  if ((e.ctrlKey || e.metaKey) && e.key === 's') { e.preventDefault(); handleExport(); }
  if (groups.length === 0) return;
  const fi = getFilteredIndices();
  if (e.key === 'ArrowDown' || e.key === 'j') { e.preventDefault(); const ci = fi.indexOf(selectedGroup); if (ci < fi.length - 1) { selectedGroup = fi[ci + 1]; editingCell = null; renderEntryList(); renderComparison(); } }
  if (e.key === 'ArrowUp' || e.key === 'k') { e.preventDefault(); const ci = fi.indexOf(selectedGroup); if (ci > 0) { selectedGroup = fi[ci - 1]; editingCell = null; renderEntryList(); renderComparison(); } }
  if (e.key === 'p') { showPasswords = !showPasswords; renderComparison(); }
  if (e.key === 't') { const cf = getConflictingFields(groups[selectedGroup]); if (cf.size > 0) { showAllFields = !showAllFields; renderComparison(); } }
});

// ─── Render ───
function updateView() {
  const has = files.length > 0;
  $('#empty-state').style.display = has ? 'none' : 'flex';
  $('#main-layout').style.display = has ? 'flex' : 'none';
  $('#btn-export').disabled = !has;
  const fb = $('#file-badge'); fb.style.display = has ? '' : 'none'; fb.textContent = `${files.length} file(s)`;
  $('#file-chips').style.display = has ? '' : 'none';
  $('#search').style.display = has ? '' : 'none';
  renderChips(); renderEntryList(); renderComparison();
}

function renderChips() {
  const list = $('#chip-list'); list.innerHTML = '';
  files.forEach((file, i) => {
    const chip = document.createElement('div');
    chip.className = 'chip ' + FILE_COLORS[i % FILE_COLORS.length];
    chip.innerHTML = `<span class="chip-name">${esc(file.name)}</span><span class="chip-count">(${file.entries.length})</span><span class="chip-close" data-idx="${i}">×</span>`;
    chip.querySelector('.chip-close').addEventListener('click', async () => {
      await invoke('remove_file', { idx: i });
      files = await invoke('get_files'); groups = await invoke('get_groups');
      if (selectedGroup >= groups.length) selectedGroup = Math.max(0, groups.length - 1);
      toast('Removed file'); updateView();
    });
    list.appendChild(chip);
  });
}

function renderEntryList() {
  const el = $('#entry-list'); el.innerHTML = '';
  const filtered = getFilteredIndices();
  if (filtered.length === 0) { el.innerHTML = '<p style="padding:20px;color:var(--gray);font-size:12px;text-align:center">No entries</p>'; return; }
  filtered.forEach(i => {
    const g = groups[i]; const isSel = i === selectedGroup;
    let bCls = 'badge-synced', bTxt = 'Synced';
    if (g.resolved_source !== null) { bCls = 'badge-resolved'; bTxt = 'Resolved'; }
    else if (g.has_conflicts) { bCls = 'badge-conflicts'; bTxt = 'Conflicts'; }
    let info = '';
    if (g.has_conflicts) info = `<div class="entry-card-info entry-card-conflicts">⚠ ${g.conflict_count || 0} differ</div>`;
    else if (g.resolved_source !== null) info = `<div class="entry-card-info entry-card-resolved">✓ Resolved</div>`;
    const card = document.createElement('div');
    card.className = 'entry-card' + (isSel ? ' selected' : '');
    card.innerHTML = `<div class="entry-card-header"><span class="entry-card-title">${esc(g.title)}</span><span class="badge ${bCls}">${bTxt}</span></div>${g.username ? `<div class="entry-card-user">${esc(g.username)}</div>` : ''}${info}`;
    card.addEventListener('click', () => { selectedGroup = i; editingCell = null; renderEntryList(); renderComparison(); });
    el.appendChild(card);
  });
}

function renderComparison() {
  const panel = $('#compare-panel');
  if (groups.length === 0) { panel.innerHTML = '<div class="empty-compare">No entries</div>'; return; }
  const g = groups[selectedGroup];
  if (!g) { panel.innerHTML = '<div class="empty-compare">No entries</div>'; return; }

  const conflicts = g.has_conflicts;
  const resolved = g.resolved_source !== null;
  const numFiles = g.entries.length;
  const cf = getConflictingFields(g);

  const seen = {};
  const fileLabels = g.entries.map(([fi]) => {
    const name = files[fi]?.name || `File ${fi}`;
    const nth = (seen[fi] = (seen[fi] || 0) + 1);
    const total = g.entries.filter(([i]) => i === fi).length;
    return total > 1 ? `${name} (${nth})` : name;
  });

  const stdKeys = new Set(['title','username','password','url','notes','created_at','updated_at','created','updated','login_username','login_password','login_uri','login_name','name','website','uri','note','extra','grouping','folder','favorite','favourite','fav','type','fields','reprompt','totp','timecreated','timepasswordchanged','timelastused','last_modified','group','email','createtime','modifytime','vault']);
  const firstEntry = g.entries[0]?.[1];
  const extras = firstEntry ? Object.keys(firstEntry.raw || {}).filter(k => !stdKeys.has(k)) : [];

  let h = '';

  // Header
  h += `<div class="compare-header"><h2>${esc(g.title)}</h2>`;
  if (g.username) h += `<span class="username">(${esc(g.username)})</span>`;
  if (conflicts) h += `<span class="badge badge-conflicts">⚠ Conflicts</span>`;
  else if (numFiles > 1) h += `<span class="badge badge-synced">✓ Synced</span>`;
  h += `<div class="right">`;
  if (numFiles > 1) {
    h += `<button class="btn" id="toggle-pw" title="P">${icon(showPasswords ? 'eye-off' : 'eye')} Passwords</button>`;
    if (cf.size > 0) {
      h += `<button class="btn" id="toggle-fields" title="T">${showAllFields ? icon('filter') + ' Diff only' : icon('list') + ' All fields'}</button>`;
    }
  }
  h += `</div></div>`;

  // Grid
  h += `<div class="compare-grid">`;
  g.entries.forEach(([fileIdx, entry], gi) => {
    const isRes = resolved && gi === g.resolved_source;
    const isSelBox = isRes && conflicts;
    let boxCls = 'entry-box' + (isSelBox ? ' resolved-selected' : '');

    h += `<div class="${boxCls}">`;
    h += `<div class="entry-box-header">`;
    if (isSelBox) h += icon('check', 'check-icon');
    h += `<span class="${FILE_COLORS[fileIdx % FILE_COLORS.length]}" style="display:inline-block;width:8px;height:8px;border-radius:50%;margin-right:4px"></span>`;
    h += `${esc(fileLabels[gi])}</div>`;

    const fieldsToShow = (showAllFields || cf.size === 0) ? FIELDS : FIELDS.filter(f => cf.has(f));

    fieldsToShow.forEach(field => {
      const val = entry[field.toLowerCase().replace(' ', '_')] || '';
      const isEditing = editingCell && editingCell.groupIdx === selectedGroup && editingCell.fileIdx === gi && editingCell.field === field;
      const isConflictField = cf.has(field);

      let rowCls = 'field-row';
      if (isConflictField && conflicts) rowCls += ' conflict';
      if (isSelBox && isConflictField) rowCls += ' resolved';

      let valueHtml = '';
      if (isEditing) {
        valueHtml = `
          <input type="text" class="edit-input" id="edit-input"
            data-group="${selectedGroup}" data-file="${gi}" data-field="${field}" value="${esc(val)}">
          <button class="edit-action-btn" data-group="${selectedGroup}" data-file="${gi}" data-field="${field}" title="Save">${icon('check')}</button>
          <button class="edit-action-btn edit-cancel" title="Cancel">${icon('x')}</button>`;
      } else {
        let displayVal = val;
        let valCls = 'field-value';
        if (field === 'Password' && !showPasswords && val) displayVal = '••••••••';
        else if (!val) { displayVal = '—'; valCls += ' empty'; }
        else if (isSelBox) valCls += ' resolved-val';
        else if (resolved) valCls += ' dimmed';

        let actionBtn = '';
        if (['Username', 'Password', 'URL', 'Notes'].includes(field)) {
          actionBtn = `<button class="edit-action-btn" data-group="${selectedGroup}" data-file="${gi}" data-field="${field}" title="Edit">${icon('pencil')}</button>`;
        }
        valueHtml = `<span class="${valCls}">${esc(displayVal)}</span>${actionBtn}`;
      }

      h += `<div class="${rowCls}">
        <span class="field-label">${icon(FIELD_ICONS[field])} ${field}</span>
        ${valueHtml}
      </div>`;
    });

    h += `<button class="keep-btn" data-group="${selectedGroup}" data-file="${fileIdx}">${icon('check')} Keep</button></div>`;
  });
  h += `</div>`;

  // Extra fields
  if (extras.length > 0) {
    h += `<div class="extra-fields-toggle" id="extra-toggle">${icon('paperclip')} Extra fields (${extras.length})<span style="margin-left:auto">▶</span></div>`;
    h += `<div class="extra-fields-content" id="extra-content" style="display:none">`;
    extras.forEach(key => {
      h += `<div class="extra-field-row"><span class="extra-field-name">${esc(key)}</span>`;
      g.entries.forEach(([, e]) => { h += `<span style="flex:1;text-align:right">${esc(e.raw?.[key] || '—')}</span>`; });
      h += `</div>`;
    });
    h += `</div>`;
  }

  // Resolved footer
  if (resolved && numFiles > 1) {
    const ri = g.entries.findIndex(([i]) => i === g.resolved_source);
    const rl = ri >= 0 ? fileLabels[ri] : '';
    h += `<div class="resolved-footer"><span>Resolved from</span><span class="resolved-source">${esc(rl)}</span><button class="clear-btn" id="clear-resolve">Clear</button></div>`;
  }

  panel.innerHTML = h;
  refreshIcons();
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
      renderEntryList(); renderComparison();
    });
  });

  // Edit action buttons - pencil (start edit) or check (save) depending on mode
  document.querySelectorAll('.edit-action-btn').forEach(btn => {
    btn.addEventListener('mousedown', (e) => {
      e.preventDefault();
      // If this is a pencil button (has data-group), start editing
      if (btn.dataset.group !== undefined && !btn.classList.contains('edit-cancel')) {
        editingCell = { groupIdx: +btn.dataset.group, fileIdx: +btn.dataset.file, field: btn.dataset.field };
        renderComparison();
        setTimeout(() => { const inp = document.getElementById('edit-input'); if (inp) { inp.focus(); inp.select(); } }, 0);
      }
    });
  });

  // Save (check) button - delegated click on compare panel
  $('#compare-panel').addEventListener('click', async (e) => {
    const btn = e.target.closest('.edit-action-btn:not(.edit-cancel)');
    if (!btn) return;
    // Only handle if we're in edit mode (check button appears)
    if (!editingCell) return;
    e.preventDefault();
    const inp = document.getElementById('edit-input');
    if (!inp) return;
    await invoke('edit_field', { groupIdx: +inp.dataset.group, entryFileIdx: +inp.dataset.file, field: inp.dataset.field, value: inp.value });
    groups = await invoke('get_groups');
    editingCell = null;
    renderEntryList(); renderComparison();
  });

  // Cancel button
  document.querySelectorAll('.edit-cancel').forEach(btn => {
    btn.addEventListener('click', () => { editingCell = null; renderComparison(); });
  });

  // Enter/Escape on inputs
  const inp = document.getElementById('edit-input');
  if (inp) {
    inp.addEventListener('keydown', async (e) => {
      e.stopPropagation();
      if (e.key === 'Enter') {
        await invoke('edit_field', { groupIdx: +inp.dataset.group, entryFileIdx: +inp.dataset.file, field: inp.dataset.field, value: inp.value });
        groups = await invoke('get_groups');
        editingCell = null;
        renderEntryList(); renderComparison();
      }
      if (e.key === 'Escape') { editingCell = null; renderComparison(); }
    });
  }

  $('#extra-toggle')?.addEventListener('click', () => {
    const c = $('#extra-content'); const vis = c.style.display !== 'none';
    c.style.display = vis ? 'none' : '';
    $('#extra-toggle span:last-child').textContent = vis ? '▶' : '▼';
  });

  $('#clear-resolve')?.addEventListener('click', async () => {
    await invoke('clear_resolve', { groupIdx: selectedGroup });
    groups = await invoke('get_groups');
    toast('Resolve cleared');
    renderEntryList(); renderComparison();
  });
}
