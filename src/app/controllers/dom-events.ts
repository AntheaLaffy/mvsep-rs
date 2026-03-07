import { openUrl } from '@tauri-apps/plugin-opener';
import { applyTheme, getThemeList } from '../../themes';
import { setLocale, t, type Locale } from '../../i18n';
import type { DomEventsContext } from '../contracts/app-context';

export function handleDocumentClick(app: DomEventsContext, e: Event): void {
  const target = app.getTargetElement(e);
  if (!target) return;

  if (target.closest('[data-nav]')) {
    const nav = target.closest('[data-nav]') as HTMLElement;
    const page = nav.dataset.nav;
    if (page) app.navigate(page);
  }

  if (target.closest('[data-action="select-file"]')) {
    void app.selectFile();
    return;
  }

  if (target.closest('[data-action="start-separation"]')) void app.startSeparation();
  if (target.closest('[data-action="start-run-workflow"]')) void app.runSeparationWorkflow();

  if (target.closest('[data-action="select-theme"]')) {
    const themeEl = target.closest('[data-theme-id]') as HTMLElement;
    const themeId = themeEl?.dataset.themeId;
    if (themeId) {
      const theme = getThemeList().find(item => item.id === themeId);
      if (theme) {
        applyTheme(theme);
        app.initializeCustomThemeDraft();
        app.render();
      }
    }
  }

  if (target.closest('[data-action="select-locale"]')) {
    const localeEl = target.closest('[data-locale]') as HTMLElement;
    const locale = localeEl?.dataset.locale as Locale | undefined;
    if (locale) {
      setLocale(locale);
      app.render();
    }
  }

  if (target.closest('[data-action="download-task"]')) {
    const taskEl = target.closest('[data-task-hash]') as HTMLElement;
    const hash = taskEl?.dataset.taskHash;
    if (hash) {
      const selectedIndex = app.selectedDownloadFileIndexByHash.get(hash) ?? null;
      app.downloadTask(hash, selectedIndex);
    }
  }

  if (target.closest('[data-action="cancel-download"]')) {
    const taskEl = target.closest('[data-task-hash]') as HTMLElement;
    const hash = taskEl?.dataset.taskHash;
    if (hash) void app.cancelDownload(hash);
  }

  if (target.closest('[data-action="delete-task"]')) {
    const taskEl = target.closest('[data-task-hash]') as HTMLElement;
    const hash = taskEl?.dataset.taskHash;
    if (hash) app.deleteTask(hash);
  }

  if (target.closest('[data-action="toggle-task-details"]')) {
    const taskEl = target.closest('[data-task-hash]') as HTMLElement;
    const hash = taskEl?.dataset.taskHash;
    if (hash) {
      if (app.expandedTaskHashes.has(hash)) app.expandedTaskHashes.delete(hash);
      else app.expandedTaskHashes.add(hash);
      app.render();
    }
  }

  if (target.closest('[data-action="test-connection"]')) void app.testConnection();
  if (target.closest('[data-action="choose-output-dir"]')) void app.chooseOutputDir();
  if (target.closest('[data-action="save-settings"]')) void app.saveSettings();
  if (target.closest('[data-action="dismiss-api-key-guide"]')) app.dismissApiKeyGuide();
  if (target.closest('[data-action="go-api-key-settings"]')) app.goToApiKeySettings();
  if (target.closest('[data-action="toggle-token-visibility"]')) app.toggleTokenVisibility();
  if (target.closest('[data-action="show-api-help"]')) app.showApiTokenHelp();

  if (target.closest('[data-action="switch-log-type"]')) {
    const type = target.closest('[data-action="switch-log-type"]')?.getAttribute('data-type') as 'frontend' | 'backend' | null;
    if (type) app.switchLogType(type);
  }

  if (target.closest('[data-action="clear-logs"]')) app.clearLogs();
  if (target.closest('[data-action="export-logs"]')) app.exportLogs();
  if (target.closest('[data-action="copy-logs"]')) app.copyLogs();

  if (target.closest('[data-action="clear-history"]')) {
    if (!app.confirmAction('Clear all task history records? This cannot be undone.')) return;
    app.clearHistory();
    app.render();
  }

  if (target.closest('[data-action="set-task-filter"]')) {
    const filter = target.closest('[data-action="set-task-filter"]')?.getAttribute('data-filter');
    if (filter === 'inProgress' || filter === 'completed' || filter === 'history') {
      app.taskViewFilter = filter;
      app.render();
    }
  }

  if (target.closest('[data-action="delete-history"]')) {
    const id = target.closest('[data-action="delete-history"]')?.getAttribute('data-id');
    if (id) {
      if (!app.confirmAction('Delete this history record? This cannot be undone.')) return;
      app.deleteFromHistory(id);
      app.render();
    }
  }

  if (target.closest('[data-action="retry-task"]')) {
    const id = target.closest('[data-action="retry-task"]')?.getAttribute('data-id');
    if (id) void app.retryFromHistory(id);
  }

  if (target.closest('[data-action="open-output"]')) {
    const path = target.closest('[data-action="open-output"]')?.getAttribute('data-path');
    if (path) void app.openOutput(path);
  }

  if (target.closest('[data-action="open-history-output"]')) {
    const id = target.closest('[data-action="open-history-output"]')?.getAttribute('data-id');
    if (id) void app.openHistoryOutput(id);
  }

  if (target.closest('[data-action="toggle-algo-details"]')) {
    const idAttr = target.closest('[data-action="toggle-algo-details"]')?.getAttribute('data-id');
    if (idAttr) {
      const algorithmId = parseInt(idAttr, 10);
      const willExpand = app.expandedAlgorithmId !== algorithmId;
      app.expandedAlgorithmId = willExpand ? algorithmId : null;
      if (willExpand) void app.loadAlgorithmDetails(algorithmId).then(() => app.render());
      else app.render();
    }
  }

  if (target.closest('[data-action="use-algorithm"]')) {
    const idAttr = target.closest('[data-action="use-algorithm"]')?.getAttribute('data-id');
    if (idAttr) {
      app.selectedAlgorithm = parseInt(idAttr, 10);
      app.selectedOpt1 = null;
      app.selectedOpt2 = null;
      app.selectedOpt3 = null;
      app.navigate('home');
      void app.loadAlgorithmDetails(app.selectedAlgorithm).then(() => app.render());
    }
  }

  if (target.closest('[data-action="refresh-algo-list"]')) {
    void app.refreshAlgorithmListFromLocal();
  }

  if (target.closest('[data-action="fetch-latest-algo-info"]')) {
    void app.fetchLatestAlgorithmInfo();
  }

  if (target.closest('[data-action="save-preset"]')) {
    const input = document.getElementById('preset-name-input') as HTMLInputElement | null;
    const name = input?.value?.trim() || app.presetNameInput.trim();
    if (!name) return;
    const preset = app.createPresetFromCurrent(name);
    app.presets.unshift(preset);
    app.selectedHomePresetId = preset.id;
    app.presetNameInput = '';
    app.savePresets();
    app.render();
  }

  if (target.closest('[data-action="apply-home-preset"]')) {
    if (app.selectedHomePresetId) void app.applyPresetById(app.selectedHomePresetId);
  }

  if (target.closest('[data-action="apply-algo-preset"]')) {
    const idAttr = target.closest('[data-action="apply-algo-preset"]')?.getAttribute('data-id');
    if (idAttr) void app.applyPresetById(idAttr);
  }

  if (target.closest('[data-action="delete-preset"]')) {
    const idAttr = target.closest('[data-action="delete-preset"]')?.getAttribute('data-id');
    if (idAttr) {
      if (!app.confirmAction('Delete this preset? This cannot be undone.')) return;
      app.presets = app.presets.filter((item: { id: string }) => item.id !== idAttr);
      if (app.selectedHomePresetId === idAttr) app.selectedHomePresetId = app.presets[0]?.id || '';
      app.savePresets();
      app.render();
    }
  }

  if (target.closest('[data-action="open-url"]')) {
    const url = target.closest('[data-action="open-url"]')?.getAttribute('data-url');
    if (url) void openUrl(url);
  }
  if (target.closest('[data-action="open-logs-page"]')) {
    app.navigate('logs');
    void app.switchLogType('backend');
    app.render();
  }

  if (target.closest('[data-action="save-custom-theme"]')) app.saveCustomTheme();

  if (target.closest('[data-action="reset-custom-theme"]')) {
    app.initializeCustomThemeDraft();
    app.render();
  }

  if (target.closest('[data-action="history-prev"]')) {
    if (app.historyPage > 0) {
      app.historyPage--;
      app.render();
    }
  }

  if (target.closest('[data-action="history-next"]')) {
    const totalPages = Math.ceil(app.getFilteredHistory().length / app.historyPageSize);
    if (app.historyPage < totalPages - 1) {
      app.historyPage++;
      app.render();
    }
  }
}

export function handleDocumentInput(app: DomEventsContext, e: Event): void {
  const target = app.getTargetElement(e) as HTMLInputElement | null;
  if (!target) return;

  if (target.id === 'log-search') {
    app.logSearchQuery = target.value;
    app.render();
  }

  if (target.id === 'algorithm-search-input') {
    app.onAlgorithmSearchInput(target.value);
  }

  if (target.id === 'history-search') {
    app.historySearchQuery = target.value;
    app.historyPage = 0;
    app.render();
  }

  if (target.id === 'custom-theme-primary' || target.id === 'custom-theme-bg' || target.id === 'custom-theme-text') {
    if (!app.customThemeDraft) app.initializeCustomThemeDraft();
    if (app.customThemeDraft) {
      if (target.id === 'custom-theme-primary') app.customThemeDraft.primary = target.value;
      if (target.id === 'custom-theme-bg') app.customThemeDraft.bgPrimary = target.value;
      if (target.id === 'custom-theme-text') app.customThemeDraft.textPrimary = target.value;
    }
  }

  if (target.id === 'preset-name-input') {
    app.presetNameInput = target.value;
  }

  if (
    target.id === 'token-input'
    || target.id === 'api-url-input'
    || target.id === 'output-dir-input'
    || target.id === 'poll-interval-input'
    || target.id === 'proxy-host-input'
    || target.id === 'proxy-port-input'
    || target.id === 'algo-auto-refresh-days-input'
  ) {
    app.scheduleSettingsAutoSave();
  }
}

export async function handleDocumentChange(app: DomEventsContext, e: Event): Promise<void> {
  const target = app.getTargetElement(e) as HTMLSelectElement | HTMLInputElement | null;
  if (!target) return;

  if (target instanceof HTMLSelectElement && target.dataset.action === 'select-download-target') {
    const hash = target.dataset.hash;
    if (hash) {
      const index = parseInt(target.value, 10);
      app.selectedDownloadFileIndexByHash.set(hash, Number.isNaN(index) || index < 0 ? null : index);
    }
    return;
  }

  if (target.id === 'log-level-filter') {
    app.logFilterLevel = target.value;
    app.render();
  }

  if (target.id === 'history-sort') {
    app.historySortOrder = target.value as 'desc' | 'asc';
    app.historyPage = 0;
    app.render();
  }

  if (target.id === 'locale-select') {
    if (app.isLoadingAlgorithmDetails) {
      app.showTransientNotice(t('algorithm.switchLocaleBlockedNotice'), 'warn', 2600);
      void app.sendDebugLog('WARN', `locale change blocked while loading algorithm details: requested=${target.value}`);
      return;
    }
    void app.sendDebugLog('INFO', `locale change requested: ${target.value}`);
    setLocale(target.value as Locale);
    app.render();
  }

  if (target.id === 'algorithm-select') {
    const value = parseInt(target.value, 10);
    app.selectedAlgorithm = value;
    app.selectedOpt1 = null;
    app.selectedOpt2 = null;
    app.selectedOpt3 = null;
    await app.loadAlgorithmDetails(value);
    app.render();
  }

  if (target.id === 'opt1-select') app.selectedOpt1 = target.value ? parseInt(target.value, 10) : null;
  if (target.id === 'opt2-select') app.selectedOpt2 = target.value ? parseInt(target.value, 10) : null;
  if (target.id === 'opt3-select') app.selectedOpt3 = target.value ? parseInt(target.value, 10) : null;
  if (target.id === 'format-select') app.selectedFormat = parseInt(target.value, 10);
  if (target.id === 'demo-checkbox') app.isDemo = (target as HTMLInputElement).checked;

  if (target.id === 'run-timeout-input') {
    app.runTimeoutSeconds = Math.max(30, parseInt(target.value || '1800', 10));
  }

  if (target.id === 'settings-output-format-select' && app.config) {
    app.config.output_format = parseInt(target.value, 10);
    app.scheduleSettingsAutoSave();
  }

  if (target.id === 'mirror-select' && app.config) {
    app.config.mirror = target.value;
    app.config.api_url = app.getApiUrlByMirror(target.value);
    const apiUrlInput = document.getElementById('api-url-input') as HTMLInputElement | null;
    if (apiUrlInput) apiUrlInput.value = app.config.api_url;
    app.scheduleSettingsAutoSave();
    app.render();
  }

  if (target.id === 'home-preset-select') app.selectedHomePresetId = target.value;
}

export function handleDocumentDragOver(app: DomEventsContext, e: DragEvent): void {
  const target = app.getTargetElement(e);
  if (!target) return;
  const dropzone = target.closest('#dropzone') as HTMLElement | null;
  if (!dropzone) return;
  e.preventDefault();
  dropzone.classList.add('active');
}

export function handleDocumentDragLeave(app: DomEventsContext, e: DragEvent): void {
  const target = app.getTargetElement(e);
  if (!target) return;
  const dropzone = target.closest('#dropzone') as HTMLElement | null;
  if (!dropzone) return;
  dropzone.classList.remove('active');
}

export function handleDocumentDrop(app: DomEventsContext, e: DragEvent): void {
  const target = app.getTargetElement(e);
  if (!target) return;
  const dropzone = target.closest('#dropzone') as HTMLElement | null;
  if (!dropzone) return;
  e.preventDefault();
  dropzone.classList.remove('active');
  const files = e.dataTransfer?.files;
  if (files && files.length > 0) app.handleFileDrop(files[0]);
}
