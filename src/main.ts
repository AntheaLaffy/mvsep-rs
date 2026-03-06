import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import { openPath, revealItemInDir } from '@tauri-apps/plugin-opener';
import { getThemeList, loadTheme } from './themes';
import { t, initLocale, getLocale, getAvailableLocales } from './i18n';
import { readJsonStorage, readTextStorage, writeJsonStorage, writeTextStorage } from './app/storage';
import { filterLogs, renderLogsPageHtml, type LogEntry } from './app/render/logs';
import { renderAboutPageHtml } from './app/render/about';
import {
  renderTasksFilterBarHtml,
  renderTasksPageHtml,
  renderTasksViewContentHtml,
} from './app/render/tasks';
import {
  renderHomeCurrentTasksSectionHtml,
  renderHomeRecentHistorySectionHtml,
} from './app/render/home';
import { renderHistoryCardHtml, renderTaskCardHtml } from './app/render/cards';
import { renderSettingsPageHtml } from './app/render/settings';
import { renderAppearancePageHtml } from './app/render/appearance';
import { renderHomePageHtml } from './app/render/home-page';
import { renderAlgorithmDetailsSectionHtml, renderAlgorithmsPageHtml } from './app/render/algorithms';
import {
  handleDocumentChange,
  handleDocumentClick,
  handleDocumentDragLeave,
  handleDocumentDragOver,
  handleDocumentDrop,
  handleDocumentInput,
} from './app/controllers/dom-events';
import {
  cancelDownload as cancelDownloadTask,
  downloadTask as downloadTaskResult,
  pollTaskStatus as pollTaskStatusTask,
  startPolling as startPollingTask,
  stopPolling as stopPollingTask,
} from './app/services/tasks';
import type { DomEventsContext, TaskServiceContext } from './app/contracts/app-context';
import type {
  AlgorithmDetails,
  AlgorithmGroup,
  Config,
  DownloadProgressPayload,
  FetchLatestAlgorithmInfoResult,
  FrontendDebugLogArgs,
  LocalAlgorithmListResponse,
  OutputFormat,
  Preset,
  SaveConfigArgs,
  Task,
  TaskHistoryRecord,
  UploadProgressPayload,
} from './app/types';

type UiActionStatusPhase = 'running' | 'success' | 'error';

interface UiStatusBanner {
  phase: UiActionStatusPhase;
  message: string;
  showLogsShortcut: boolean;
}

class App {
  private readonly defaultApiToken = 'mvsep-demo-key';
  private readonly defaultAlgorithmAutoRefreshDays = 15;
  private currentPage: string = 'home';
  private config: Config | null = null;
  private tasks: Task[] = [];
  private algorithms: AlgorithmGroup[] = [];
  private algorithmDetails: Map<number, AlgorithmDetails> = new Map();
  private formats: OutputFormat[] = [];
  private selectedFile: string | null = null;
  private selectedAlgorithm: number = 49;
  private selectedOpt1: number | null = null;
  private selectedOpt2: number | null = null;
  private selectedOpt3: number | null = null;
  private selectedFormat: number = 1;
  private isDemo: boolean = false;
  private taskPollingIntervals: Map<string, number> = new Map();
  private pollInFlightHashes: Set<string> = new Set();
  private frontendLogs: LogEntry[] = [];
  private backendLogs: LogEntry[] = [];
  private currentLogType: 'frontend' | 'backend' = 'frontend';
  private logFilterLevel: string = 'all';
  private logSearchQuery: string = '';
  private taskHistory: TaskHistoryRecord[] = [];
  private historySearchQuery: string = '';
  private historySortOrder: 'desc' | 'asc' = 'desc';
  private historyPage: number = 0;
  private historyPageSize: number = 20;
  private taskViewFilter: 'inProgress' | 'completed' | 'history' = 'inProgress';
  private algorithmSearchQuery: string = '';
  private algorithmSearchResults: AlgorithmGroup[] = [];
  private expandedAlgorithmId: number | null = null;
  private algorithmSearchDebounceTimer: number | null = null;
  private customThemeDraft: {
    primary: string;
    bgPrimary: string;
    textPrimary: string;
  } | null = null;
  private expandedTaskHashes: Set<string> = new Set();
  private queueInfo: { active: number; queued: number } | null = null;
  private runTimeoutSeconds: number = 1800;
  private connectionStatus: 'idle' | 'testing' | 'success' | 'error' = 'idle';
  private connectionStatusText: string = '';
  private algorithmCachePath: string | null = null;
  private lastAlgorithmCacheUpdatedAt: string | null = null;
  private presets: Preset[] = [];
  private selectedHomePresetId: string = '';
  private presetNameInput: string = '';
  private isSubmittingTask: boolean = false;
  private submittingAction: 'create' | 'run' | null = null;
  private isInitialLoading: boolean = true;
  private uploadFileName: string = '';
  private uploadBytes: number = 0;
  private uploadTotalBytes: number = 0;
  private uploadSpeedBps: number = 0;
  private uploadPercent: number = 0;
  private uploadFailed: boolean = false;
  private isShellMounted: boolean = false;
  private lastRenderedPage: string | null = null;
  private backendLogPollTimer: number | null = null;
  private queueInfoLastFetchAt: number = 0;
  private queueInfoInFlight: Promise<void> | null = null;
  private renderSeq: number = 0;
  private renderInProgress: boolean = false;
  private droppedRenderCount: number = 0;
  private renderScheduled: boolean = false;
  private renderPending: boolean = false;
  private lastSidebarRenderKey: string | null = null;
  private logStickToBottom: boolean = true;
  private lastRenderedLogCountByType: Record<'frontend' | 'backend', number> = {
    frontend: 0,
    backend: 0,
  };
  private isLoadingAlgorithmDetails: boolean = false;
  private transientNotice: { message: string; level: 'info' | 'warn' } | null = null;
  private transientNoticeTimer: number | null = null;
  private selectedDownloadFileIndexByHash: Map<string, number | null> = new Map();
  private cancellingDownloadHashes: Set<string> = new Set();
  private runningActions: Set<string> = new Set();
  private statusBanner: UiStatusBanner | null = null;
  private statusBannerTimer: number | null = null;
  private isTokenVisible: boolean = false;
  private activeTasksSaveTimer: number | null = null;
  private readonly activeTasksSaveDebounceMs: number = 500;
  private readonly minPollIntervalSeconds: number = 1;
  private readonly maxPollIntervalSeconds: number = 60;

  constructor() {
    this.init();
  }

  async init() {
    initLocale();
    loadTheme();
    this.applySavedCustomThemeIfSelected();
    this.initializeCustomThemeDraft();
    await this.loadConfig();
    this.algorithmSearchResults = this.algorithms;
    this.setupEventListeners();
    this.setupDownloadProgressListener();
    this.setupUploadProgressListener();
    this.initFrontendLogs();
    this.loadTaskHistory();
    this.loadActiveTasks();
    this.loadPresets();
    this.startBackendLogPolling();
    this.setupFrontendDebugHooks();
    this.render();

    void this.loadInitialData();
  }

  setupDownloadProgressListener() {
    void listen<DownloadProgressPayload>('download-progress', (event) => {
      const payload = event.payload;
      const task = this.tasks.find(t => t.hash === payload.hash);
      if (!task) return;
      task.phase = payload.done ? 'done' : 'downloading';
      task.download_file_name = payload.file_name;
      task.download_bytes = payload.downloaded_bytes;
      task.download_total_bytes = payload.total_bytes;
      task.download_speed_bps = payload.speed_bps;
      task.download_percent = payload.percent;
      if (this.shouldRenderTaskUpdates()) {
        this.requestTaskPanelsRefresh();
      }
    });
  }

  setupUploadProgressListener() {
    void listen<UploadProgressPayload>('upload-progress', (event) => {
      const payload = event.payload;
      this.uploadFileName = payload.file_name;
      this.uploadBytes = payload.uploaded_bytes;
      this.uploadTotalBytes = payload.total_bytes;
      this.uploadSpeedBps = payload.speed_bps;
      this.uploadPercent = payload.percent;
      this.uploadFailed = payload.failed;

      if (payload.done && payload.failed) {
        this.isSubmittingTask = false;
        this.submittingAction = null;
      }
      if (this.currentPage === 'home') {
        this.render();
      }
    });
  }

  setupFrontendDebugHooks() {
    window.addEventListener('error', (event) => {
      void this.sendDebugLog('ERROR', `window.error: ${event.message} @ ${event.filename}:${event.lineno}:${event.colno}`);
    });
    window.addEventListener('unhandledrejection', (event) => {
      void this.sendDebugLog('ERROR', `unhandledrejection: ${String(event.reason)}`);
    });
  }

  async sendDebugLog(level: 'INFO' | 'WARN' | 'ERROR' | 'DEBUG', message: string) {
    try {
      await invoke<void>('frontend_debug_log', { level, message } satisfies FrontendDebugLogArgs);
    } catch {
      // ignore debug bridge failures
    }
  }

  showTransientNotice(message: string, level: 'info' | 'warn' = 'warn', durationMs: number = 3000) {
    this.transientNotice = { message, level };
    if (this.transientNoticeTimer !== null) {
      clearTimeout(this.transientNoticeTimer);
    }
    this.transientNoticeTimer = window.setTimeout(() => {
      this.transientNotice = null;
      this.transientNoticeTimer = null;
      this.render();
    }, durationMs);
    this.render();
  }

  private setStatusBanner(
    phase: UiActionStatusPhase,
    message: string,
    options: { autoHideMs?: number; showLogsShortcut?: boolean } = {},
  ) {
    const { autoHideMs = 0, showLogsShortcut = phase === 'error' } = options;
    this.statusBanner = { phase, message, showLogsShortcut };
    if (this.statusBannerTimer !== null) {
      clearTimeout(this.statusBannerTimer);
      this.statusBannerTimer = null;
    }
    if (autoHideMs > 0) {
      this.statusBannerTimer = window.setTimeout(() => {
        this.statusBanner = null;
        this.statusBannerTimer = null;
        this.render();
      }, autoHideMs);
    }
    this.render();
  }

  private setActionRunning(actionKey: string, running: boolean) {
    if (running) this.runningActions.add(actionKey);
    else this.runningActions.delete(actionKey);
  }

  private isActionRunning(actionKey: string): boolean {
    return this.runningActions.has(actionKey);
  }

  private async withUiAction<T>(
    actionKey: string,
    options: {
      runningMessage?: string;
      successMessage?: string;
      errorMessage?: string;
      showLogsShortcutOnError?: boolean;
      autoHideSuccessMs?: number;
    } = {},
    runner: () => Promise<T>,
  ): Promise<T | null> {
    if (this.isActionRunning(actionKey)) {
      return null;
    }
    this.setActionRunning(actionKey, true);
    if (options.runningMessage) {
      this.setStatusBanner('running', options.runningMessage, { autoHideMs: 0, showLogsShortcut: false });
    }
    this.render();
    try {
      const result = await runner();
      if (options.successMessage) {
        this.setStatusBanner('success', options.successMessage, { autoHideMs: options.autoHideSuccessMs ?? 2200, showLogsShortcut: false });
      }
      return result;
    } catch (e) {
      const fallbackMessage = options.errorMessage || this.getErrorMessage(e);
      this.setStatusBanner('error', fallbackMessage, {
        autoHideMs: 0,
        showLogsShortcut: options.showLogsShortcutOnError !== false,
      });
      throw e;
    } finally {
      this.setActionRunning(actionKey, false);
      this.render();
    }
  }

  private getErrorMessage(error: unknown): string {
    if (error instanceof Error) return error.message;
    if (typeof error === 'string') return error;
    if (error && typeof error === 'object') {
      const maybeMessage = (error as { message?: unknown }).message;
      if (typeof maybeMessage === 'string') return maybeMessage;
      try {
        return JSON.stringify(error);
      } catch {
        return String(error);
      }
    }
    return String(error);
  }

  private isLocalCacheMissingError(message: string): boolean {
    const msg = message.toLowerCase();
    return (
      msg.includes('os error 2')
      || msg.includes('no such file or directory')
      || msg.includes('没有那个文件或目录')
      || msg.includes('cache')
    );
  }

  startBackendLogPolling() {
    if (this.backendLogPollTimer !== null) {
      clearInterval(this.backendLogPollTimer);
    }
    this.backendLogPollTimer = window.setInterval(() => {
      if (this.currentPage !== 'logs' || this.currentLogType !== 'backend') return;
      void this.loadBackendLogs().then((changed) => {
        if (changed) this.render();
      });
    }, 1200);
  }

  private shouldRenderTaskUpdates(): boolean {
    return this.currentPage === 'home' || this.currentPage === 'tasks';
  }

  private getInProgressTasks(): Task[] {
    return this.tasks.filter(t => t.status !== 'done' && t.status !== 'failed');
  }

  private getCompletedTasks(): Task[] {
    return this.tasks.filter(t => t.status === 'done');
  }

  private requestTaskPanelsRefresh() {
    if (!this.shouldRenderTaskUpdates()) return;
    if (!this.refreshTaskPanels()) {
      this.render();
    }
  }

  private refreshTaskPanels(): boolean {
    if (this.currentPage === 'home') {
      const tasksSection = document.getElementById('home-current-tasks-section');
      const historySection = document.getElementById('home-recent-history-section');
      if (!tasksSection || !historySection) return false;
      const inProgressTasks = this.getInProgressTasks();
      const taskListEl = document.getElementById('home-current-task-list');
      if (!(taskListEl && this.patchTaskCardList(taskListEl, inProgressTasks))) {
        const nextTasksHtml = this.renderHomeCurrentTasksSection();
        if (tasksSection.innerHTML !== nextTasksHtml) {
          tasksSection.innerHTML = nextTasksHtml;
        }
      }
      const nextHistoryHtml = this.renderHomeRecentHistorySection();
      if (historySection.innerHTML !== nextHistoryHtml) {
        historySection.innerHTML = nextHistoryHtml;
      }
      return true;
    }

    if (this.currentPage === 'tasks') {
      const filterBar = document.getElementById('tasks-filter-bar');
      const viewContent = document.getElementById('tasks-view-content');
      if (!filterBar || !viewContent) return false;
      const activeEl = document.activeElement as HTMLElement | null;
      const focusedId = activeEl?.id || null;
      const shouldRestoreTextFocus = activeEl instanceof HTMLInputElement || activeEl instanceof HTMLTextAreaElement;
      const selectionStart = shouldRestoreTextFocus ? activeEl.selectionStart : null;
      const selectionEnd = shouldRestoreTextFocus ? activeEl.selectionEnd : null;
      const nextFilterHtml = this.renderTasksFilterBar();
      if (filterBar.innerHTML !== nextFilterHtml) {
        filterBar.innerHTML = nextFilterHtml;
      }
      const visibleTasks = this.taskViewFilter === 'inProgress'
        ? this.getInProgressTasks()
        : this.taskViewFilter === 'completed'
          ? this.getCompletedTasks()
          : null;
      const taskListEl = document.getElementById('tasks-current-task-list');
      if (!(taskListEl && visibleTasks && this.patchTaskCardList(taskListEl, visibleTasks))) {
        const nextViewHtml = this.renderTasksViewContent();
        if (viewContent.innerHTML !== nextViewHtml) {
          viewContent.innerHTML = nextViewHtml;
        }
      }
      if (focusedId && shouldRestoreTextFocus) {
        const nextFocused = document.getElementById(focusedId) as (HTMLInputElement | HTMLTextAreaElement | null);
        if (nextFocused) {
          try {
            nextFocused.focus({ preventScroll: true });
          } catch {
            nextFocused.focus();
          }
          if (selectionStart !== null && selectionEnd !== null) {
            try {
              nextFocused.setSelectionRange(selectionStart, selectionEnd);
            } catch {
              // ignore unsupported input types
            }
          }
        }
      }
      return true;
    }

    return false;
  }

  private patchTaskCardList(container: HTMLElement, tasks: Task[]): boolean {
    const existingCards = Array.from(container.children).filter((el) =>
      el instanceof HTMLElement && !!el.getAttribute('data-task-hash'),
    ) as HTMLElement[];
    if (existingCards.length !== tasks.length) return false;

    for (let i = 0; i < tasks.length; i++) {
      const currentEl = existingCards[i];
      if ((currentEl.getAttribute('data-task-hash') || '') !== tasks[i].hash) {
        return false;
      }
    }

    for (let i = 0; i < tasks.length; i++) {
      const currentEl = existingCards[i];
      const nextHtml = this.renderTaskCard(tasks[i]).trim();
      if (currentEl.outerHTML !== nextHtml) {
        const temp = document.createElement('div');
        temp.innerHTML = nextHtml;
        const nextEl = temp.firstElementChild;
        if (!nextEl) return false;
        currentEl.replaceWith(nextEl);
      }
    }

    return true;
  }

  private isSameQueueInfo(
    a: { active: number; queued: number } | null,
    b: { active: number; queued: number } | null,
  ): boolean {
    if (!a && !b) return true;
    if (!a || !b) return false;
    return a.active === b.active && a.queued === b.queued;
  }

  private renderQueueInfoBadge(): string {
    if (!this.queueInfo) return '';
    return `
      <div class="text-sm px-3 py-1 rounded-full bg-bg-secondary border border-border text-text-secondary">
        Queue: ${this.queueInfo.active} active / ${this.queueInfo.queued} queued
      </div>
    `;
  }

  private refreshHeaderQueueInfo(): boolean {
    const slot = document.getElementById('queue-info-slot');
    if (!slot) return false;
    const nextHtml = this.renderQueueInfoBadge();
    if (slot.innerHTML !== nextHtml) {
      slot.innerHTML = nextHtml;
    }
    return true;
  }

  async loadInitialData() {
    this.isInitialLoading = true;
    this.render();

    await this.loadAlgorithmCachePath();
    await this.refreshAlgorithmListFromLocal({ render: false, syncSearch: true }).catch(() => undefined);

    const startupTasks: Array<Promise<unknown>> = [this.loadFormats()];
    if (this.shouldLoadQueueInfoForPage(this.currentPage)) {
      startupTasks.push(this.loadQueueInfo(true));
    }
    await Promise.allSettled(startupTasks);

    if (this.shouldAutoRefreshAlgorithmCache()) {
      await this.fetchLatestAlgorithmInfo({ render: false, showNotice: false });
    }

    this.isInitialLoading = false;
    this.render();
  }

  private shouldLoadQueueInfoForPage(page: string): boolean {
    return page === 'home' || page === 'tasks';
  }

  getTargetElement(event: Event): HTMLElement | null {
    const raw = event.target;
    if (!raw) return null;
    if (raw instanceof HTMLElement) return raw;
    if (raw instanceof Element) return raw as HTMLElement;
    if (raw instanceof Node && raw.parentElement) return raw.parentElement;
    return null;
  }

  getPresetStorageKey() {
    return 'mvsep_presets_v1';
  }

  loadPresets() {
    this.presets = readJsonStorage<Preset[]>(this.getPresetStorageKey(), []);
    this.selectedHomePresetId = this.presets[0]?.id || '';
  }

  savePresets() {
    const ok = writeJsonStorage(this.getPresetStorageKey(), this.presets);
    if (!ok) {
      console.error('Failed to save presets');
    }
  }

  createPresetFromCurrent(name: string): Preset {
    return {
      id: `preset_${Date.now()}`,
      name: name.trim(),
      algorithmId: this.selectedAlgorithm,
      opt1: this.selectedOpt1,
      opt2: this.selectedOpt2,
      opt3: this.selectedOpt3,
      formatId: this.selectedFormat,
      demo: this.isDemo,
    };
  }

  async applyPresetById(presetId: string) {
    const preset = this.presets.find(p => p.id === presetId);
    if (!preset) return;
    this.selectedAlgorithm = preset.algorithmId;
    this.selectedOpt1 = preset.opt1;
    this.selectedOpt2 = preset.opt2;
    this.selectedOpt3 = preset.opt3;
    this.selectedFormat = preset.formatId;
    this.isDemo = preset.demo;
    this.render();
  }

  async loadQueueInfo(force: boolean = false) {
    if (!this.config?.token || !this.config?.api_url) return;
    const apiUrl = this.config.api_url;
    const token = this.config.token;
    const now = Date.now();
    if (!force && now - this.queueInfoLastFetchAt < 4000) return;
    if (this.queueInfoInFlight) return this.queueInfoInFlight;

    this.queueInfoInFlight = (async () => {
      const previous = this.queueInfo;
      try {
        const info = await invoke<{ active: number; queued: number }>('get_queue_info', {
          apiUrl,
          token,
        });
        this.queueInfo = info;
        this.queueInfoLastFetchAt = Date.now();
        if (!this.isSameQueueInfo(previous, this.queueInfo)) {
          if (!this.refreshHeaderQueueInfo()) {
            this.render();
          } else {
            this.requestTaskPanelsRefresh();
          }
        }
      } catch (e) {
        console.error('Failed to load queue info:', e);
        this.queueInfo = null;
        if (!this.isSameQueueInfo(previous, this.queueInfo)) {
          if (!this.refreshHeaderQueueInfo()) {
            this.render();
          } else {
            this.requestTaskPanelsRefresh();
          }
        }
      } finally {
        this.queueInfoInFlight = null;
      }
    })();

    return this.queueInfoInFlight;
  }

  // ============== Task History Methods ==============

  getActiveTasksStorageKey() {
    return 'mvsep_active_tasks_v1';
  }

  loadActiveTasks() {
    try {
      const parsed = readJsonStorage<Task[]>(this.getActiveTasksStorageKey(), []);
      this.tasks = parsed.filter(t => t.status !== 'done' && t.status !== 'failed');
      for (const task of this.tasks) {
        task.phase = this.getPhaseFromStatus(task.status);
      }
      for (const task of this.tasks) {
        this.startPolling(task.hash);
        void this.pollTaskStatus(task.hash);
      }
    } catch (e) {
      console.error('Failed to load active tasks:', e);
      this.tasks = [];
    }
  }

  saveActiveTasks(force: boolean = false) {
    if (!force) {
      if (this.activeTasksSaveTimer !== null) {
        clearTimeout(this.activeTasksSaveTimer);
      }
      this.activeTasksSaveTimer = window.setTimeout(() => {
        this.activeTasksSaveTimer = null;
        this.saveActiveTasks(true);
      }, this.activeTasksSaveDebounceMs);
      return;
    }

    if (this.activeTasksSaveTimer !== null) {
      clearTimeout(this.activeTasksSaveTimer);
      this.activeTasksSaveTimer = null;
    }

    const activeTasks = this.tasks.filter(t => t.status !== 'done' && t.status !== 'failed');
    const ok = writeJsonStorage(this.getActiveTasksStorageKey(), activeTasks);
    if (!ok) {
      console.error('Failed to save active tasks');
    }
  }

  loadTaskHistory() {
    this.taskHistory = readJsonStorage<TaskHistoryRecord[]>('mvsep_task_history', []);
  }

  saveTaskHistory() {
    // Keep only last 100 records
    if (this.taskHistory.length > 100) {
      this.taskHistory = this.taskHistory.slice(-100);
    }
    const ok = writeJsonStorage('mvsep_task_history', this.taskHistory);
    if (!ok) {
      console.error('Failed to save task history');
    }
  }

  addToHistory(task: Task, outputPath: string | null = null) {
    const format = this.formats.find(f => f.id === task.format);
    const record: TaskHistoryRecord = {
      id: task.hash,
      fileName: task.file_name,
      algorithmId: task.algorithm_id,
      algorithmName: task.algorithm_name,
      modelId: task.model_id,
      modelName: task.model_name,
      model2Id: task.model2_id ?? null,
      model2Name: task.model2_name ?? null,
      model3Id: task.model3_id ?? null,
      model3Name: task.model3_name ?? null,
      formatId: task.format,
      formatName: format?.name || 'Unknown',
      status: task.status as 'done' | 'failed',
      createdAt: task.created_at,
      completedAt: Date.now(),
      outputFiles: task.output_files,
      outputPath: outputPath,
      error: task.error,
    };
    const existingIndex = this.taskHistory.findIndex(item => item.id === record.id);
    if (existingIndex >= 0) {
      this.taskHistory.splice(existingIndex, 1);
    }
    this.taskHistory.unshift(record);
    this.saveTaskHistory();
  }

  deleteFromHistory(id: string) {
    this.taskHistory = this.taskHistory.filter(t => t.id !== id);
    this.saveTaskHistory();
  }

  clearHistory() {
    this.taskHistory = [];
    this.saveTaskHistory();
  }

  initializeCustomThemeDraft() {
    const styles = getComputedStyle(document.documentElement);
    this.customThemeDraft = {
      primary: styles.getPropertyValue('--color-primary').trim() || '#0891B2',
      bgPrimary: styles.getPropertyValue('--color-bg-primary').trim() || '#ECFEFF',
      textPrimary: styles.getPropertyValue('--color-text-primary').trim() || '#164E63',
    };
  }

  applySavedCustomThemeIfSelected() {
    const selected = readTextStorage('theme');
    if (selected !== 'custom') return;
    const custom = readJsonStorage<{ primary: string; bgPrimary: string; textPrimary: string } | null>(
      'mvsep_custom_theme',
      null,
    );
    if (!custom) return;
    document.documentElement.style.setProperty('--color-primary', custom.primary);
    document.documentElement.style.setProperty('--color-primary-light', custom.primary);
    document.documentElement.style.setProperty('--color-cta', custom.primary);
    document.documentElement.style.setProperty('--color-bg-primary', custom.bgPrimary);
    document.documentElement.style.setProperty('--color-text-primary', custom.textPrimary);
    document.documentElement.setAttribute('data-theme', 'custom');
    this.customThemeDraft = custom;
  }

  getFilteredHistory() {
    let filtered = [...this.taskHistory];

    // Search filter
    if (this.historySearchQuery) {
      const query = this.historySearchQuery.toLowerCase();
      filtered = filtered.filter(t =>
        t.fileName.toLowerCase().includes(query) ||
        t.algorithmName.toLowerCase().includes(query)
      );
    }

    // Sort
    filtered.sort((a, b) => {
      const timeA = a.completedAt || a.createdAt;
      const timeB = b.completedAt || b.createdAt;
      return this.historySortOrder === 'desc' ? timeB - timeA : timeA - timeB;
    });

    return filtered;
  }

  getPaginatedHistory() {
    const filtered = this.getFilteredHistory();
    const start = this.historyPage * this.historyPageSize;
    return filtered.slice(start, start + this.historyPageSize);
  }

  async retryFromHistory(id: string) {
    const record = this.taskHistory.find(t => t.id === id);
    if (!record) return;

    // Refill algorithm and all model options from history
    this.selectedAlgorithm = record.algorithmId;
    this.selectedOpt1 = record.modelId;
    this.selectedOpt2 = record.model2Id ?? null;
    this.selectedOpt3 = record.model3Id ?? null;
    this.selectedFormat = record.formatId;

    // Navigate to home page
    this.navigate('home');
    await this.loadAlgorithmDetails(record.algorithmId);
    this.validateSelectedOptionsForCurrentAlgorithm();
    this.render();
    this.addFrontendLog(
      'INFO',
      `Retrying task: ${record.fileName}. Refilled algorithm=${record.algorithmName}, opt1=${record.modelName || '-'}, opt2=${record.model2Name || '-'}, opt3=${record.model3Name || '-'}, format=${record.formatName}. Please select source audio on Home page.`
    );
  }

  initFrontendLogs() {
    // Add initial log
    this.addFrontendLog('INFO', 'Application initialized');
    // Override console methods to capture logs
    const originalLog = console.log;
    const originalWarn = console.warn;
    const originalError = console.error;

    console.log = (...args) => {
      originalLog.apply(console, args);
      this.addFrontendLog('INFO', args.map(a => String(a)).join(' '));
    };
    console.warn = (...args) => {
      originalWarn.apply(console, args);
      this.addFrontendLog('WARN', args.map(a => String(a)).join(' '));
    };
    console.error = (...args) => {
      originalError.apply(console, args);
      this.addFrontendLog('ERROR', args.map(a => String(a)).join(' '));
    };
  }

  addFrontendLog(level: string, message: string) {
    const entry = {
      timestamp: new Date().toLocaleString(),
      level,
      message,
    };
    this.frontendLogs.push(entry);
    // Keep only last 1000 entries
    if (this.frontendLogs.length > 1000) {
      this.frontendLogs = this.frontendLogs.slice(-1000);
    }
  }

  private hasLogListChanged(
    prev: LogEntry[],
    next: LogEntry[],
  ): boolean {
    if (prev.length !== next.length) return true;
    for (let i = 0; i < prev.length; i++) {
      const a = prev[i];
      const b = next[i];
      if (!a || !b) return true;
      if (a.timestamp !== b.timestamp || a.level !== b.level || a.message !== b.message) {
        return true;
      }
    }
    return false;
  }

  async loadBackendLogs(): Promise<boolean> {
    try {
      const logs = await invoke<LogEntry[]>('get_backend_logs');
      const changed = this.hasLogListChanged(this.backendLogs, logs);
      this.backendLogs = logs;
      return changed;
    } catch (e) {
      console.error('Failed to load backend logs:', e);
      return false;
    }
  }

  async loadConfig() {
    try {
      this.config = await invoke<Config>('load_config');
      if (!this.config.token || !this.config.token.trim()) {
        this.config.token = this.defaultApiToken;
      }
      if (this.config.mirror && (!this.config.api_url || this.config.api_url.includes('mvsep.com'))) {
        this.config.api_url = this.getApiUrlByMirror(this.config.mirror);
      }
      this.config.algorithm_auto_refresh_days = this.normalizeAlgorithmAutoRefreshDays(this.config.algorithm_auto_refresh_days);
      this.selectedFormat = this.config.output_format ?? 1;
    } catch (e) {
      console.error('Failed to load config:', e);
      this.config = {
        token: this.defaultApiToken,
        api_url: 'https://mvsep.com',
        mirror: 'main',
        proxy_mode: 'system',
        proxy_host: '127.0.0.1',
        proxy_port: '7897',
        output_dir: './output',
        output_format: 1,
        poll_interval: 5,
        algorithm_auto_refresh_days: this.defaultAlgorithmAutoRefreshDays,
      };
      this.selectedFormat = 1;
    }
  }

  async loadAlgorithmCachePath() {
    try {
      this.algorithmCachePath = await invoke<string>('get_algorithm_cache_path_cmd');
    } catch (e) {
      console.error('Failed to load algorithm cache path:', e);
      this.algorithmCachePath = null;
    }
  }

  async saveConfig(config: Config) {
    try {
      await invoke<void>('save_config', { config } satisfies SaveConfigArgs);
      this.config = config;
    } catch (e) {
      console.error('Failed to save config:', e);
    }
  }

  private filterAlgorithmGroupsFromLocal(query: string): AlgorithmGroup[] {
    const keyword = query.trim().toLowerCase();
    if (!keyword) return this.algorithms;
    const groups = this.algorithms
      .map((group) => ({
        ...group,
        algorithms: group.algorithms.filter((algo) => algo.name.toLowerCase().includes(keyword)),
      }))
      .filter((group) => group.algorithms.length > 0);
    return groups;
  }

  private getFirstAvailableAlgorithmId(groups: AlgorithmGroup[]): number | null {
    for (const group of groups) {
      for (const algo of group.algorithms) {
        return algo.id;
      }
    }
    return null;
  }

  private hasAlgorithmId(groups: AlgorithmGroup[], algorithmId: number): boolean {
    return groups.some((group) => group.algorithms.some((algo) => algo.id === algorithmId));
  }

  private applyAlgorithmSearchFilter() {
    this.algorithmSearchResults = this.filterAlgorithmGroupsFromLocal(this.algorithmSearchQuery);
  }

  async refreshAlgorithmListFromLocal(options: { render?: boolean; syncSearch?: boolean } = {}) {
    const shouldRender = options.render !== false;
    const syncSearch = options.syncSearch !== false;
    try {
      await this.withUiAction(
        'algo-refresh-list',
        {
          runningMessage: t('algorithm.refreshListRunning'),
          successMessage: t('algorithm.cacheRefreshed'),
          errorMessage: t('algorithm.refreshListFailed'),
        },
        async () => {
          const result = await invoke<LocalAlgorithmListResponse>('refresh_algorithm_list_from_local');
          this.lastAlgorithmCacheUpdatedAt = result.updated_at || null;
          this.algorithms = result.groups;
          if (syncSearch) {
            this.applyAlgorithmSearchFilter();
          } else {
            this.algorithmSearchResults = result.groups;
          }
          const validIds = new Set(this.algorithms.flatMap((group) => group.algorithms.map((algo) => algo.id)));
          for (const id of this.algorithmDetails.keys()) {
            if (!validIds.has(id)) {
              this.algorithmDetails.delete(id);
            }
          }

          const previousAlgorithm = this.selectedAlgorithm;
          if (!this.hasAlgorithmId(this.algorithms, this.selectedAlgorithm)) {
            const fallback = this.getFirstAvailableAlgorithmId(this.algorithms);
            if (fallback !== null) {
              this.selectedAlgorithm = fallback;
              this.selectedOpt1 = null;
              this.selectedOpt2 = null;
              this.selectedOpt3 = null;
              if (previousAlgorithm !== fallback) {
                this.showTransientNotice(`Algorithm ${previousAlgorithm} is unavailable. Switched to ${fallback}.`, 'warn', 3200);
              }
            }
          }

          if (this.selectedAlgorithm && !this.algorithmDetails.has(this.selectedAlgorithm)) {
            await this.loadAlgorithmDetails(this.selectedAlgorithm);
          }
          this.validateSelectedOptionsForCurrentAlgorithm();
        },
      );
    } catch (e) {
      console.error('Failed to refresh algorithm list from local cache:', e);
      const message = this.getErrorMessage(e);
      if (this.isLocalCacheMissingError(message)) {
        this.showTransientNotice(t('algorithm.cacheMissing'), 'warn', 3400);
      } else {
        this.showTransientNotice(message, 'warn', 3400);
      }
    } finally {
      if (shouldRender) this.render();
    }
  }

  async fetchLatestAlgorithmInfo(options: { render?: boolean; showNotice?: boolean } = {}) {
    if (!this.config?.token || !this.config?.api_url) {
      await this.refreshAlgorithmListFromLocal({ render: options.render !== false, syncSearch: true });
      return;
    }
    const shouldRender = options.render !== false;
    const showNotice = options.showNotice !== false;

    this.isLoadingAlgorithmDetails = true;
    if (shouldRender) this.render();
    try {
      await this.withUiAction(
        'algo-fetch-latest',
        {
          runningMessage: t('algorithm.fetchLatestRunning'),
          successMessage: showNotice ? t('algorithm.latestFetched') : '',
          errorMessage: t('algorithm.fetchLatestFailed'),
        },
        async () => {
          await invoke<FetchLatestAlgorithmInfoResult>('fetch_latest_algorithm_info', {
            apiUrl: this.config?.api_url,
            token: this.config?.token,
            proxyMode: this.config?.proxy_mode,
            proxyHost: this.config?.proxy_host,
            proxyPort: this.config?.proxy_port,
          });
          await this.refreshAlgorithmListFromLocal({ render: false, syncSearch: true });
          if (this.selectedAlgorithm) {
            await this.loadAlgorithmDetails(this.selectedAlgorithm);
          }
          this.validateSelectedOptionsForCurrentAlgorithm();
        },
      );
    } catch (e) {
      console.error('Failed to fetch latest algorithm info:', e);
      await this.refreshAlgorithmListFromLocal({ render: false, syncSearch: true });
      const message = this.getErrorMessage(e);
      this.showTransientNotice(message, 'warn', 3200);
    } finally {
      this.isLoadingAlgorithmDetails = false;
      if (shouldRender) this.render();
    }
  }

  async loadFormats() {
    if (!this.config?.token || !this.config?.api_url) {
      this.formats = [
        { id: 0, name: 'MP3 (320 kbps)' },
        { id: 1, name: 'WAV (16 bit)' },
        { id: 2, name: 'FLAC (16 bit)' },
        { id: 3, name: 'M4A (lossy)' },
        { id: 4, name: 'WAV (32 bit)' },
        { id: 5, name: 'FLAC (24 bit)' },
      ];
      return;
    }
    
    try {
      this.formats = await invoke<OutputFormat[]>('list_formats', {
        apiUrl: this.config.api_url,
        token: this.config.token,
      });
    } catch (e) {
      console.error('Failed to load formats:', e);
      this.formats = [
        { id: 0, name: 'MP3 (320 kbps)' },
        { id: 1, name: 'WAV (16 bit)' },
        { id: 2, name: 'FLAC (16 bit)' },
        { id: 3, name: 'M4A (lossy)' },
        { id: 4, name: 'WAV (32 bit)' },
        { id: 5, name: 'FLAC (24 bit)' },
      ];
    }
  }

  async loadAlgorithmDetails(algorithmId: number): Promise<AlgorithmDetails | null> {
    if (algorithmId <= 0) return null;
    void this.sendDebugLog('INFO', `loadAlgorithmDetails start from local cache: id=${algorithmId}`);
    try {
      const details = await invoke<AlgorithmDetails>('get_algorithm_details_from_local', {
        algorithmId,
      });
      this.algorithmDetails.set(algorithmId, details);
      void this.sendDebugLog('INFO', `loadAlgorithmDetails done: id=${algorithmId}, fields=${details.fields.length}`);
      return details;
    } catch (e) {
      console.error('Failed to load algorithm details:', e);
      const message = this.getErrorMessage(e);
      if (this.isLocalCacheMissingError(message)) {
        this.showTransientNotice(t('algorithm.cacheMissing'), 'warn', 3600);
      } else {
        this.showTransientNotice(message, 'warn', 3200);
      }
      void this.sendDebugLog('ERROR', `loadAlgorithmDetails failed: id=${algorithmId}, err=${message}`);
      return null;
    }
  }

  onAlgorithmSearchInput(query: string) {
    this.algorithmSearchQuery = query;

    if (this.algorithmSearchDebounceTimer) {
      clearTimeout(this.algorithmSearchDebounceTimer);
      this.algorithmSearchDebounceTimer = null;
    }

    this.algorithmSearchDebounceTimer = window.setTimeout(() => {
      this.applyAlgorithmSearchFilter();
      this.render();
    }, 300);
  }

  setupEventListeners() {
    const domCtx = this as unknown as DomEventsContext;
    document.addEventListener('click', (e) => handleDocumentClick(domCtx, e));
    document.addEventListener('input', (e) => handleDocumentInput(domCtx, e));
    document.addEventListener('change', (e) => {
      void handleDocumentChange(domCtx, e);
    });
    document.addEventListener('dragover', (e) => handleDocumentDragOver(domCtx, e));
    document.addEventListener('dragleave', (e) => handleDocumentDragLeave(domCtx, e));
    document.addEventListener('drop', (e) => handleDocumentDrop(domCtx, e));
  }

  navigate(page: string) {
    this.currentPage = page;
    if (this.shouldLoadQueueInfoForPage(page)) {
      void this.loadQueueInfo(false);
    }
    this.render();
  }

  async selectFile() {
    try {
      const selected = await openDialog({
        multiple: false,
        filters: [{
          name: 'Audio',
          extensions: ['wav', 'mp3', 'flac', 'ogg', 'm4a', 'aac'],
        }],
      });
      
      if (selected) {
        this.selectedFile = selected as string;
        this.render();
      }
    } catch (e) {
      console.error('Failed to select file:', e);
    }
  }

  handleFileDrop(file: File) {
    const pseudoPath = (file as File & { path?: string }).path;
    this.selectedFile = pseudoPath || file.name;
    this.render();
  }

  async startSeparation() {
    if (this.isSubmittingTask) return;
    void this.sendDebugLog('INFO', 'startSeparation requested');
    this.setStatusBanner('running', t('home.createPending'), { showLogsShortcut: false });
    const hash = await this.createTaskFromCurrentSelection('create');
    if (!hash) return;
    this.startPolling(hash);
    this.addFrontendLog('INFO', `Task created: ${hash}. Background polling started.`);
    void this.sendDebugLog('INFO', `startSeparation created task: ${hash}`);
    this.setStatusBanner('success', t('home.createTaskSucceeded'), { autoHideMs: 2200, showLogsShortcut: false });
    this.render();
  }

  async runSeparationWorkflow() {
    if (this.isSubmittingTask) return;
    void this.sendDebugLog('INFO', 'runSeparationWorkflow requested');
    this.setStatusBanner('running', t('home.runPending'), { showLogsShortcut: false });
    const hash = await this.createTaskFromCurrentSelection('run');
    if (!hash) return;
    this.addFrontendLog('INFO', `One-click run started: ${hash}`);
    void this.sendDebugLog('INFO', `runSeparationWorkflow created task: ${hash}`);

    try {
      const completed = await this.waitForTaskCompletion(hash, this.runTimeoutSeconds);
      if (this.isSuccessStatus(completed.status)) {
        this.addFrontendLog('INFO', `Task completed, starting auto download: ${hash}`);
        void this.sendDebugLog('INFO', `runSeparationWorkflow task completed, auto download start: ${hash}`);
        await this.downloadTask(hash);
        this.setStatusBanner('success', t('home.runSucceeded'), { autoHideMs: 2400, showLogsShortcut: false });
      } else {
        this.addFrontendLog('WARN', `One-click run ended with non-success status: ${completed.status}`);
        void this.sendDebugLog('WARN', `runSeparationWorkflow non-success status: ${completed.status}`);
        this.setStatusBanner('error', t('home.runFailed'), { showLogsShortcut: true });
      }
    } catch (e) {
      console.error('Run workflow failed:', e);
      this.addFrontendLog('ERROR', `One-click run failed: ${String(e)}`);
      void this.sendDebugLog('ERROR', `runSeparationWorkflow failed: ${String(e)}`);
      this.setStatusBanner('error', t('home.runFailed'), { showLogsShortcut: true });
    } finally {
      this.render();
    }
  }

  async createTaskFromCurrentSelection(action: 'create' | 'run'): Promise<string | null> {
    if (this.isSubmittingTask) return null;
    if (!this.selectedFile || !this.config) return null;
    if (!this.selectedFile.includes('/') && !this.selectedFile.includes('\\')) {
      alert('Drag-and-drop did not expose file path. Please use file picker.');
      return null;
    }

    const details = this.algorithmDetails.get(this.selectedAlgorithm);
    const hasOpt1 = details?.fields.some(f => f.name === 'add_opt1');
    if (hasOpt1 && this.selectedOpt1 === null) {
      alert('Please select an option for --opt1');
      return null;
    }

    this.isSubmittingTask = true;
    this.submittingAction = action;
    this.uploadFileName = this.selectedFile.split('/').pop() || this.selectedFile.split('\\').pop() || 'audio';
    this.uploadBytes = 0;
    this.uploadTotalBytes = 0;
    this.uploadSpeedBps = 0;
    this.uploadPercent = 0;
    this.uploadFailed = false;
    this.addFrontendLog('INFO', `${action === 'run' ? 'Run' : 'Create'} task started`);
    void this.sendDebugLog('INFO', `createTaskFromCurrentSelection start: action=${action}, algorithm=${this.selectedAlgorithm}`);
    this.render();

    try {
      const hash = await invoke<string>('create_task', {
        filePath: this.selectedFile,
        sepType: this.selectedAlgorithm,
        opt1: this.selectedOpt1,
        opt2: this.selectedOpt2,
        opt3: this.selectedOpt3,
        outputFormat: this.selectedFormat,
        demo: this.isDemo,
        apiUrl: this.config.api_url,
        token: this.config.token,
      });

      const task: Task = {
        hash,
        file_name: this.selectedFile.split('/').pop() || 'unknown',
        algorithm_id: this.selectedAlgorithm,
        algorithm_name: this.getAlgorithmName(this.selectedAlgorithm),
        model_id: this.selectedOpt1,
        model_name: this.getSelectedModelName(),
        model2_id: this.selectedOpt2,
        model2_name: this.getSelectedOptionName('add_opt2', this.selectedOpt2),
        model3_id: this.selectedOpt3,
        model3_name: this.getSelectedOptionName('add_opt3', this.selectedOpt3),
        format: this.selectedFormat,
        status: 'waiting',
        progress: 0,
        created_at: Date.now(),
        output_files: [],
        error: null,
        message: null,
        queue_count: null,
        current_order: null,
        phase: 'queueing',
        download_file_name: null,
        download_bytes: 0,
        download_total_bytes: null,
        download_speed_bps: 0,
        download_percent: 0,
      };

      this.tasks.push(task);
      this.saveActiveTasks();
      void this.sendDebugLog('INFO', `createTaskFromCurrentSelection success: hash=${hash}`);
      this.render();
      return hash;
    } catch (e) {
      console.error('Failed to create task:', e);
      this.uploadFailed = true;
      void this.sendDebugLog('ERROR', `createTaskFromCurrentSelection failed: ${String(e)}`);
      this.setStatusBanner('error', t('home.createTaskFailed'), { showLogsShortcut: true });
      return null;
    } finally {
      this.isSubmittingTask = false;
      this.submittingAction = null;
      this.render();
    }
  }

  async waitForTaskCompletion(hash: string, timeoutSeconds: number): Promise<Task> {
    const intervalSeconds = this.getPollIntervalSeconds();
    const startedAt = Date.now();

    while (true) {
      await this.pollTaskStatus(hash);
      const task = this.tasks.find(t => t.hash === hash);
      if (!task) {
        throw new Error(`Task ${hash} not found`);
      }
      if (this.isTerminalStatus(task.status)) {
        return task;
      }
      if ((Date.now() - startedAt) / 1000 > timeoutSeconds) {
        this.stopPolling(hash);
        task.status = 'failed';
        task.error = `Timeout after ${timeoutSeconds}s`;
        this.addToHistory(task, this.config?.output_dir || null);
        throw new Error(task.error);
      }
      await new Promise(resolve => setTimeout(resolve, intervalSeconds * 1000));
    }
  }

  getAlgorithmName(id: number): string {
    for (const group of this.algorithms) {
      const algo = group.algorithms.find(a => a.id === id);
      if (algo) return algo.name;
    }
    return `Algorithm ${id}`;
  }

  getSelectedModelName(): string | null {
    if (this.selectedOpt1 === null) return null;
    const details = this.algorithmDetails.get(this.selectedAlgorithm);
    const opt1Field = details?.fields.find(f => f.name === 'add_opt1');
    if (!opt1Field) return null;
    return opt1Field.options[String(this.selectedOpt1)] ?? null;
  }

  normalizeTaskStatus(status: string): string {
    const s = (status || '').toLowerCase();
    if (s === 'done' || s === 'success' || s === 'completed' || s === 'complete' || s === 'finished') {
      return 'done';
    }
    if (s === 'failed' || s === 'error' || s === 'cancelled' || s === 'canceled' || s === 'timeout') {
      return 'failed';
    }
    return status;
  }

  isTerminalStatus(status: string): boolean {
    const normalized = this.normalizeTaskStatus(status);
    return normalized === 'done' || normalized === 'failed';
  }

  isSuccessStatus(status: string): boolean {
    return this.normalizeTaskStatus(status) === 'done';
  }

  getSelectedOptionName(fieldName: 'add_opt2' | 'add_opt3', value: number | null): string | null {
    if (value === null) return null;
    const details = this.algorithmDetails.get(this.selectedAlgorithm);
    const field = details?.fields.find(f => f.name === fieldName);
    if (!field) return null;
    return field.options[String(value)] ?? null;
  }

  validateSelectedOptionsForCurrentAlgorithm() {
    const details = this.algorithmDetails.get(this.selectedAlgorithm);
    if (!details) return;
    const opt1Field = details.fields.find(f => f.name === 'add_opt1');
    const opt2Field = details.fields.find(f => f.name === 'add_opt2');
    const opt3Field = details.fields.find(f => f.name === 'add_opt3');
    if (this.selectedOpt1 !== null && (!opt1Field || !(String(this.selectedOpt1) in opt1Field.options))) {
      this.selectedOpt1 = null;
    }
    if (this.selectedOpt2 !== null && (!opt2Field || !(String(this.selectedOpt2) in opt2Field.options))) {
      this.selectedOpt2 = null;
    }
    if (this.selectedOpt3 !== null && (!opt3Field || !(String(this.selectedOpt3) in opt3Field.options))) {
      this.selectedOpt3 = null;
    }
  }

  getPhaseFromStatus(status: string): Task['phase'] {
    const normalized = this.normalizeTaskStatus(status);
    const lowered = normalized.toLowerCase();
    if (
      lowered === 'waiting' ||
      lowered === 'queued' ||
      lowered === 'queue' ||
      lowered === 'pending' ||
      lowered === 'distributing'
    ) return 'queueing';
    if (
      lowered === 'processing' ||
      lowered === 'running' ||
      lowered === 'working' ||
      lowered === 'merging'
    ) return 'separating';
    if (normalized === 'done') return 'done';
    if (normalized === 'failed') return 'failed';
    return 'queueing';
  }

  formatBytes(bytes: number | null | undefined): string {
    if (bytes === null || bytes === undefined) return '-';
    if (bytes < 1024) return `${bytes} B`;
    const kb = bytes / 1024;
    if (kb < 1024) return `${kb.toFixed(1)} KB`;
    const mb = kb / 1024;
    if (mb < 1024) return `${mb.toFixed(1)} MB`;
    const gb = mb / 1024;
    return `${gb.toFixed(2)} GB`;
  }

  getDisplayFileName(pathOrName: string): string {
    const slash = pathOrName.lastIndexOf('/');
    const backslash = pathOrName.lastIndexOf('\\');
    const idx = Math.max(slash, backslash);
    const base = idx >= 0 ? pathOrName.slice(idx + 1) : pathOrName;
    return base || pathOrName;
  }

  startPolling(hash: string) {
    this.touchTaskPollingState();
    startPollingTask(this as unknown as TaskServiceContext, hash);
  }

  async pollTaskStatus(hash: string) {
    this.touchTaskPollingState();
    await pollTaskStatusTask(this as unknown as TaskServiceContext, hash);
  }

  stopPolling(hash: string) {
    this.touchTaskPollingState();
    stopPollingTask(this as unknown as TaskServiceContext, hash);
  }

  private touchTaskPollingState() {
    void this.taskPollingIntervals;
    void this.pollInFlightHashes;
  }

  async downloadTask(hash: string, fileIndex: number | null = null) {
    await downloadTaskResult(this as unknown as TaskServiceContext, hash, fileIndex);
  }

  async cancelDownload(hash: string) {
    await cancelDownloadTask(this as unknown as TaskServiceContext, hash);
  }

  deleteTask(hash: string) {
    const task = this.tasks.find(t => t.hash === hash);
    if (task) {
      const phase = task.phase || this.getPhaseFromStatus(task.status);
      const canDelete = (task.status === 'done' || task.status === 'failed') && phase !== 'downloading';
      if (!canDelete) {
        this.showTransientNotice('Only completed or failed tasks can be deleted.', 'warn', 2600);
        return;
      }
    }
    if (!this.confirmAction('Delete this task from local list? This cannot be undone.')) return;
    this.stopPolling(hash);
    this.tasks = this.tasks.filter(t => t.hash !== hash);
    this.saveActiveTasks(true);
    this.requestTaskPanelsRefresh();
  }

  async openOutput(path: string): Promise<boolean> {
    const normalized = await this.resolveToAbsolutePath(path);
    this.addFrontendLog('INFO', `Open output requested: ${normalized}`);
    void this.sendDebugLog('INFO', `openOutput requested: ${normalized}`);
    const isLinux = /linux/i.test(navigator.userAgent);
    try {
      await invoke<void>('open_in_file_manager', { path: normalized });
      this.addFrontendLog('INFO', `Open output succeeded: ${normalized}`);
      void this.sendDebugLog('INFO', `openOutput succeeded via backend: ${normalized}`);
      return true;
    } catch (e) {
      this.addFrontendLog('WARN', `open_in_file_manager failed, trying fallback: ${String(e)}`);
      void this.sendDebugLog('WARN', `openOutput backend open failed: ${String(e)}`);
      if (isLinux) {
        if (this.config?.output_dir) {
          try {
            const fallback = await this.resolveToAbsolutePath(this.config.output_dir);
            await invoke<void>('open_in_file_manager', { path: fallback });
            this.addFrontendLog('INFO', `Open output fallback succeeded: ${fallback}`);
            void this.sendDebugLog('INFO', `openOutput fallback succeeded via backend: ${fallback}`);
            return true;
          } catch {
            // ignore and let error log below handle
          }
        }
        console.error('Failed to open output location:', e);
        void this.sendDebugLog('ERROR', `openOutput failed on linux: ${String(e)}`);
        return false;
      }
      try {
        await revealItemInDir(normalized);
        this.addFrontendLog('INFO', `Reveal in dir succeeded: ${normalized}`);
        void this.sendDebugLog('INFO', `openOutput succeeded via revealItemInDir: ${normalized}`);
        return true;
      } catch (err) {
        try {
          await openPath(normalized);
          this.addFrontendLog('INFO', `openPath succeeded: ${normalized}`);
          void this.sendDebugLog('INFO', `openOutput succeeded via openPath: ${normalized}`);
          return true;
        } catch (_) {
          // ignored, fall through
        }
        if (this.config?.output_dir) {
          try {
            const fallback = await this.resolveToAbsolutePath(this.config.output_dir);
            await invoke<void>('open_in_file_manager', { path: fallback });
            void this.sendDebugLog('INFO', `openOutput fallback succeeded via backend: ${fallback}`);
            return true;
          } catch (_) {
            // ignored, fall through to final error log
          }
        }
        console.error('Failed to open output location:', e, err);
        void this.sendDebugLog('ERROR', `openOutput failed after all fallbacks: ${String(e)} | ${String(err)}`);
        return false;
      }
    }
  }

  async openHistoryOutput(recordId: string) {
    const record = this.taskHistory.find(r => r.id === recordId);
    if (!record) return;
    const candidates: string[] = [];
    if (record.outputPath) {
      candidates.push(record.outputPath);
    }
    if (record.outputFiles.length > 0) {
      candidates.push(this.getParentPath(record.outputFiles[0]));
    }
    if (this.config?.output_dir) {
      candidates.push(this.config.output_dir);
    }
    const unique = [...new Set(candidates.filter(Boolean))];
    for (const path of unique) {
      const ok = await this.openOutput(path);
      if (ok) return;
    }
  }

  getParentPath(path: string): string {
    const normalized = path.replace(/\\/g, '/');
    const idx = normalized.lastIndexOf('/');
    if (idx <= 0) return path;
    return normalized.slice(0, idx);
  }

  async resolveToAbsolutePath(path: string): Promise<string> {
    if (path.startsWith('/') || /^[a-zA-Z]:[\\/]/.test(path)) {
      return path;
    }
    try {
      const absolute = await invoke<string>('resolve_path', { path });
      return absolute;
    } catch {
      return path;
    }
  }

  saveCustomTheme() {
    if (!this.customThemeDraft) return;
    const draft = this.customThemeDraft;
    document.documentElement.style.setProperty('--color-primary', draft.primary);
    document.documentElement.style.setProperty('--color-primary-light', draft.primary);
    document.documentElement.style.setProperty('--color-cta', draft.primary);
    document.documentElement.style.setProperty('--color-bg-primary', draft.bgPrimary);
    document.documentElement.style.setProperty('--color-text-primary', draft.textPrimary);
    document.documentElement.setAttribute('data-theme', 'custom');
    const themeSaved = writeTextStorage('theme', 'custom');
    const customSaved = writeJsonStorage('mvsep_custom_theme', draft);
    if (!themeSaved || !customSaved) {
      console.error('Failed to persist custom theme');
    }
    this.addFrontendLog('INFO', 'Custom theme saved');
    this.render();
  }

  getApiUrlByMirror(mirror: string): string {
    return mirror === 'mirror' ? 'https://mirror.mvsep.com' : 'https://mvsep.com';
  }

  async chooseOutputDir() {
    try {
      await this.withUiAction(
        'settings-choose-output',
        {
          runningMessage: t('settings.choosingFolder'),
          successMessage: t('settings.outputDirUpdated'),
          errorMessage: t('settings.chooseFolderFailed'),
        },
        async () => {
          const selected = await openDialog({
            directory: true,
            multiple: false,
          });
          if (!selected || !this.config) return;
          this.config.output_dir = selected as string;
        },
      );
    } catch (e) {
      console.error('Failed to choose output directory:', e);
    } finally {
      this.render();
    }
  }

  async testConnection() {
    if (!this.config?.token || !this.config?.api_url) return;
    this.connectionStatus = 'testing';
    this.connectionStatusText = t('common.loading');
    this.render();

    try {
      const result = await this.withUiAction(
        'settings-test-connection',
        {
          runningMessage: t('settings.testing'),
          successMessage: t('settings.connectionSucceeded'),
          errorMessage: t('settings.connectionFailed'),
        },
        async () => invoke<boolean>('test_connection', {
          token: this.config?.token,
          apiUrl: this.config?.api_url,
        }),
      );
      if (result === null) return;
      this.connectionStatus = result ? 'success' : 'error';
      this.connectionStatusText = result ? t('settings.connected') : t('settings.disconnected');
    } catch (e) {
      this.connectionStatus = 'error';
      this.connectionStatusText = t('settings.disconnected');
    }
    this.render();
  }

  async saveSettings() {
    const tokenInput = document.getElementById('token-input') as HTMLInputElement;
    const apiUrlInput = document.getElementById('api-url-input') as HTMLInputElement;
    const outputDirInput = document.getElementById('output-dir-input') as HTMLInputElement;
    const pollIntervalInput = document.getElementById('poll-interval-input') as HTMLInputElement;
    const proxyModeSelect = document.getElementById('proxy-mode-select') as HTMLSelectElement;
    const proxyHostInput = document.getElementById('proxy-host-input') as HTMLInputElement;
    const proxyPortInput = document.getElementById('proxy-port-input') as HTMLInputElement;
    const mirrorSelect = document.getElementById('mirror-select') as HTMLSelectElement;
    const outputFormatSelect = document.getElementById('settings-output-format-select') as HTMLSelectElement;
    const algorithmAutoRefreshDaysInput = document.getElementById('algo-auto-refresh-days-input') as HTMLInputElement;

    if (!this.config) return;
    try {
      await this.withUiAction(
        'settings-save',
        {
          runningMessage: t('settings.saving'),
          successMessage: t('settings.saved'),
          errorMessage: t('settings.saveFailed'),
        },
        async () => {
          this.config!.token = tokenInput?.value?.trim() || this.defaultApiToken;
          this.config!.api_url = apiUrlInput?.value || 'https://mvsep.com';
          this.config!.output_dir = outputDirInput?.value || './output';
          const requestedPoll = parseInt(pollIntervalInput?.value || '5', 10);
          const normalizedPoll = this.normalizePollInterval(requestedPoll);
          this.config!.poll_interval = normalizedPoll;
          this.config!.proxy_mode = proxyModeSelect?.value || 'system';
          this.config!.proxy_host = proxyHostInput?.value || '127.0.0.1';
          this.config!.proxy_port = proxyPortInput?.value || '7897';
          this.config!.mirror = mirrorSelect?.value || 'main';
          this.config!.output_format = outputFormatSelect?.value ? parseInt(outputFormatSelect.value, 10) : 1;
          const requestedAutoRefreshDays = parseInt(algorithmAutoRefreshDaysInput?.value || String(this.defaultAlgorithmAutoRefreshDays), 10);
          const normalizedAutoRefreshDays = this.normalizeAlgorithmAutoRefreshDays(requestedAutoRefreshDays);
          this.config!.algorithm_auto_refresh_days = normalizedAutoRefreshDays;
          this.selectedFormat = this.config!.output_format ?? this.selectedFormat;
          if (pollIntervalInput) {
            pollIntervalInput.value = String(normalizedPoll);
          }
          if (requestedPoll !== normalizedPoll) {
            this.showTransientNotice(
              `Poll interval adjusted to ${normalizedPoll}s (allowed: ${this.minPollIntervalSeconds}-${this.maxPollIntervalSeconds}s).`,
              'warn',
              3200,
            );
          }
          if (algorithmAutoRefreshDaysInput) {
            algorithmAutoRefreshDaysInput.value = String(normalizedAutoRefreshDays);
          }
          await this.saveConfig(this.config!);
          await this.loadFormats();
        },
      );
      this.showTransientNotice('Settings saved. Click "Fetch Latest Algorithm Info" to update local algorithm cache.', 'info', 3200);
    } catch (e) {
      console.error('Failed to save settings:', e);
    } finally {
      this.render();
    }
  }

  toggleTokenVisibility() {
    this.isTokenVisible = !this.isTokenVisible;
    void this.sendDebugLog('INFO', `token visibility toggled: ${this.isTokenVisible ? 'visible' : 'hidden'}`);
    this.render();
  }

  showApiTokenHelp() {
    this.showTransientNotice(t('settings.apiKeyHelpText'), 'info', 6200);
    this.setStatusBanner('running', t('settings.apiKeyHelpShown'), { autoHideMs: 2600, showLogsShortcut: false });
    void this.sendDebugLog('INFO', 'api token help requested');
  }

  private normalizePollInterval(value: number | null | undefined): number {
    if (!Number.isFinite(value)) return 5;
    const rounded = Math.round(value as number);
    return Math.min(this.maxPollIntervalSeconds, Math.max(this.minPollIntervalSeconds, rounded));
  }

  private getPollIntervalSeconds(): number {
    return this.normalizePollInterval(this.config?.poll_interval ?? 5);
  }

  private normalizeAlgorithmAutoRefreshDays(value: number | null | undefined): number {
    if (!Number.isFinite(value)) return this.defaultAlgorithmAutoRefreshDays;
    const rounded = Math.round(value as number);
    return Math.min(365, Math.max(1, rounded));
  }

  private parseCacheUpdatedAtToMs(updatedAt: string | null): number | null {
    if (!updatedAt) return null;
    const asNumber = Number(updatedAt);
    if (Number.isFinite(asNumber) && asNumber > 0) {
      return asNumber * 1000;
    }
    const fromDate = Date.parse(updatedAt);
    if (Number.isFinite(fromDate) && fromDate > 0) {
      return fromDate;
    }
    return null;
  }

  private shouldAutoRefreshAlgorithmCache(): boolean {
    if (!this.config?.token || !this.config?.api_url) return false;
    const days = this.normalizeAlgorithmAutoRefreshDays(this.config.algorithm_auto_refresh_days);
    const cacheUpdatedAtMs = this.parseCacheUpdatedAtToMs(this.lastAlgorithmCacheUpdatedAt);
    if (cacheUpdatedAtMs === null) return true;
    const ttlMs = days * 24 * 60 * 60 * 1000;
    return Date.now() - cacheUpdatedAtMs >= ttlMs;
  }

  private confirmAction(message: string): boolean {
    return window.confirm(message);
  }

  render() {
    this.renderPending = true;
    if (this.renderScheduled) return;
    this.renderScheduled = true;
    window.requestAnimationFrame(() => {
      this.renderScheduled = false;
      if (!this.renderPending) return;
      this.renderPending = false;
      this.performRender();
      if (this.renderPending) {
        this.render();
      }
    });
  }

  private performRender() {
    const app = document.getElementById('app');
    if (!app) return;
    if (this.renderInProgress) {
      this.droppedRenderCount += 1;
      this.renderPending = true;
      return;
    }
    this.renderInProgress = true;
    const renderId = ++this.renderSeq;
    const startedAt = performance.now();
    try {
      const activeEl = document.activeElement as HTMLElement | null;
      const focusedId = activeEl?.id || null;
      const shouldRestoreTextFocus = activeEl instanceof HTMLInputElement || activeEl instanceof HTMLTextAreaElement;
      const selectionStart = shouldRestoreTextFocus ? activeEl.selectionStart : null;
      const selectionEnd = shouldRestoreTextFocus ? activeEl.selectionEnd : null;

      if (!this.isShellMounted || !document.getElementById('main-content')) {
        app.innerHTML = `
          <div class="flex h-screen">
            <div id="sidebar-root"></div>
            <div id="main-content" class="flex-1 md:ml-56 p-4 md:p-6 pb-24 md:pb-6 overflow-y-auto"></div>
          </div>
        `;
        this.isShellMounted = true;
      }

      const sidebarRoot = document.getElementById('sidebar-root');
      if (sidebarRoot) {
        const sidebarKey = `${this.currentPage}|${getLocale()}`;
        if (this.lastSidebarRenderKey !== sidebarKey) {
          sidebarRoot.innerHTML = this.renderSidebar();
          this.lastSidebarRenderKey = sidebarKey;
        }
      }

      const prevLogContainer = this.currentPage === 'logs'
        ? document.getElementById('log-container')
        : null;
      if (prevLogContainer) {
        const distanceFromBottom = prevLogContainer.scrollHeight - prevLogContainer.scrollTop - prevLogContainer.clientHeight;
        this.logStickToBottom = distanceFromBottom <= 32;
      }

      const mainContent = document.getElementById('main-content');
      if (!mainContent) return;
      const preserveScroll = this.lastRenderedPage === this.currentPage;
      const previousScrollTop = mainContent.scrollTop;
      mainContent.innerHTML = `
        ${this.transientNotice ? `
          <div class="mb-4 px-3 py-2 rounded-lg border ${
            this.transientNotice.level === 'warn'
              ? 'bg-yellow-50 border-yellow-200 text-yellow-800'
              : 'bg-blue-50 border-blue-200 text-blue-800'
          } text-sm">
            ${this.escapeHtml(this.transientNotice.message)}
          </div>
        ` : ''}
        ${this.renderHeader()}
        ${this.renderStatusBanner()}
        ${this.renderPage()}
      `;
      if (preserveScroll) {
        mainContent.scrollTop = previousScrollTop;
      } else {
        mainContent.scrollTop = 0;
      }

      // Restore focus only for text inputs to avoid select-related event loops on some WebKit/Wayland setups.
      if (focusedId && shouldRestoreTextFocus) {
        const nextFocused = document.getElementById(focusedId) as (HTMLInputElement | HTMLTextAreaElement | null);
        if (nextFocused) {
          try {
            nextFocused.focus({ preventScroll: true });
          } catch {
            try {
              nextFocused.focus();
            } catch {
              // ignore focus restore failures on specific webview engines
            }
          }
          if (selectionStart !== null && selectionEnd !== null) {
            try {
              const inputType = nextFocused instanceof HTMLInputElement ? nextFocused.type : 'text';
              const selectionSafeTypes = ['text', 'search', 'url', 'tel', 'password'];
              if (!(nextFocused instanceof HTMLInputElement) || selectionSafeTypes.includes(inputType)) {
                nextFocused.setSelectionRange(selectionStart, selectionEnd);
              }
            } catch {
              // ignore selection restore failures for unsupported input types
            }
          }
        }
      }

      this.lastRenderedPage = this.currentPage;
      if (this.currentPage === 'logs') {
        const logContainer = document.getElementById('log-container');
        if (logContainer) {
          const renderedCount = this.getFilteredLogs().length;
          const previousCount = this.lastRenderedLogCountByType[this.currentLogType] ?? 0;
          const hasNewLogs = renderedCount > previousCount;
          if (hasNewLogs && this.logStickToBottom) {
            logContainer.scrollTop = logContainer.scrollHeight;
          }
          this.lastRenderedLogCountByType[this.currentLogType] = renderedCount;
        }
      }
    } catch (e) {
      console.error('Render failed:', e);
      void this.sendDebugLog('ERROR', `render failed: ${String(e)} (page=${this.currentPage}, renderId=${renderId})`);
      app.innerHTML = `
        <div class="p-6">
          <div class="card">
            <h3 class="font-semibold text-red-500 mb-2">Render Error</h3>
            <p class="text-sm text-text-muted">UI render failed. Check frontend logs for details.</p>
          </div>
        </div>
      `;
    } finally {
      const cost = performance.now() - startedAt;
      if (cost > 120) {
        void this.sendDebugLog(
          'WARN',
          `slow render: id=${renderId}, page=${this.currentPage}, cost_ms=${cost.toFixed(1)}, dropped=${this.droppedRenderCount}`
        );
      } else if (this.droppedRenderCount > 0) {
        void this.sendDebugLog(
          'DEBUG',
          `render recovered: id=${renderId}, page=${this.currentPage}, dropped=${this.droppedRenderCount}`
        );
      }
      this.droppedRenderCount = 0;
      this.renderInProgress = false;
    }
  }

  renderSidebar() {
    const navItems = [
      { id: 'home', icon: '🏠', label: t('nav.home') },
      { id: 'tasks', icon: '🎵', label: t('nav.tasks') },
      { id: 'algorithms', icon: '🔍', label: t('nav.algorithms') },
      { id: 'settings', icon: '⚙️', label: t('nav.settings') },
      { id: 'appearance', icon: '🎨', label: t('nav.appearance') },
      { id: 'logs', icon: '📋', label: t('nav.logs') },
      { id: 'about', icon: 'ℹ️', label: t('nav.about') },
    ];

    return `
      <aside class="sidebar">
        <div class="p-4 border-b border-border">
          <h1 class="text-xl font-bold text-primary">MVSEP</h1>
          <p class="text-xs text-text-muted">${t('app.title')}</p>
        </div>
        <nav class="flex-1 py-2">
          ${navItems.map(item => `
            <button
              type="button"
              class="sidebar-item ${this.currentPage === item.id ? 'active' : ''}"
              data-nav="${item.id}"
              aria-current="${this.currentPage === item.id ? 'page' : 'false'}"
            >
              <span>${item.icon}</span>
              <span>${item.label}</span>
            </button>
          `).join('')}
        </nav>
      </aside>
    `;
  }

  renderHeader() {
    const locales = getAvailableLocales();
    const currentLocale = getLocale();

    return `
      <div class="flex items-center justify-between mb-6">
        <div class="flex items-center gap-4">
          <h2 class="text-2xl font-bold text-text-primary">${t(`nav.${this.currentPage}`)}</h2>
          <div id="queue-info-slot">${this.renderQueueInfoBadge()}</div>
        </div>
        <div class="flex items-center gap-3">
          <select class="select w-28" id="locale-select" ${this.isLoadingAlgorithmDetails ? 'disabled' : ''}>
            ${locales.map(l => `
              <option value="${l.code}" ${currentLocale === l.code ? 'selected' : ''}>${l.nativeName}</option>
            `).join('')}
          </select>
        </div>
      </div>
    `;
  }

  renderStatusBanner() {
    if (!this.statusBanner) return '';
    const levelClasses = this.statusBanner.phase === 'error'
      ? 'bg-red-50 border-red-200 text-red-800'
      : this.statusBanner.phase === 'success'
        ? 'bg-green-50 border-green-200 text-green-800'
        : 'bg-blue-50 border-blue-200 text-blue-800';
    return `
      <div class="mb-4 px-3 py-2 rounded-lg border ${levelClasses} text-sm flex items-center justify-between gap-2">
        <span>${this.escapeHtml(this.statusBanner.message)}</span>
        ${this.statusBanner.showLogsShortcut ? `
          <button class="btn btn-secondary text-xs px-2 py-1" type="button" data-action="open-logs-page">
            ${t('logs.title')}
          </button>
        ` : ''}
      </div>
    `;
  }

  renderPage() {
    switch (this.currentPage) {
      case 'home': return this.renderHomePage();
      case 'tasks': return this.renderTasksPage();
      case 'algorithms': return this.renderAlgorithmsPage();
      case 'settings': return this.renderSettingsPage();
      case 'appearance': return this.renderAppearancePage();
      case 'logs': return this.renderLogsPage();
      case 'about': return this.renderAboutPage();
      default: return this.renderHomePage();
    }
  }

  private renderHomeCurrentTasksSection(): string {
    const currentTasks = this.getInProgressTasks();
    return renderHomeCurrentTasksSectionHtml({
      currentTasks,
      renderTaskCard: (task) => this.renderTaskCard(task),
      t,
    });
  }

  private renderHomeRecentHistorySection(): string {
    const recentHistory = this.taskHistory.slice(0, 3);
    return renderHomeRecentHistorySectionHtml({
      recentHistory,
      renderHistoryCard: (record) => this.renderHistoryCard(record),
      t,
    });
  }

  renderHomePage() {
    const selectedFileLabel = this.selectedFile
      ? this.escapeHtml(this.selectedFile.split('/').pop() || this.selectedFile)
      : '';
    const algorithmSelectOptionsHtml = this.algorithms
      .flatMap(g => g.algorithms)
      .map(a => `
        <option value="${a.id}" ${a.id === this.selectedAlgorithm ? 'selected' : ''}>
          ${this.escapeHtml(a.name)}
        </option>
      `)
      .join('');
    const presetBlockHtml = this.presets.length > 0
      ? `
        <div>
          <label class="block text-sm text-text-secondary mb-1">${t('home.presetLoadLabel')}</label>
          <div class="flex gap-2">
            <select class="select" id="home-preset-select">
              ${this.presets.map(p => `
                <option value="${p.id}" ${p.id === this.selectedHomePresetId ? 'selected' : ''}>${this.escapeHtml(p.name)}</option>
              `).join('')}
            </select>
            <button class="btn btn-secondary whitespace-nowrap" data-action="apply-home-preset">${t('common.load')}</button>
          </div>
        </div>
      `
      : '';
    const formatOptionsHtml = this.formats.map(f => `
      <option value="${f.id}" ${f.id === this.selectedFormat ? 'selected' : ''}>
        ${this.escapeHtml(f.name)}
      </option>
    `).join('');
    const uploadInfoHtml = `
      <div class="text-sm px-3 py-2 rounded-lg bg-bg-primary border border-border text-text-secondary">
        <p class="mb-1">${t('home.uploading')}: ${this.escapeHtml(this.uploadFileName || 'audio')}</p>
        <div class="progress-bar mb-1">
          <div class="progress-bar-fill" style="width: ${Math.max(0, Math.min(100, this.uploadPercent || 0))}%"></div>
        </div>
        <p>${(this.uploadPercent || 0).toFixed(1)}% · ${this.formatBytes(this.uploadBytes)} / ${this.formatBytes(this.uploadTotalBytes || null)} · ${this.formatBytes(this.uploadSpeedBps)}/s</p>
        ${this.uploadFailed ? `<p class="text-red-500 mt-1">${t('home.uploadFailed')}</p>` : ''}
      </div>
    `;

    return renderHomePageHtml({
      isInitialLoading: this.isInitialLoading,
      selectedFile: this.selectedFile,
      selectedFileLabelEscaped: selectedFileLabel,
      algorithmSelectOptionsHtml,
      algorithmOptionsHtml: this.renderAlgorithmOptions(),
      presetBlockHtml,
      formatOptionsHtml,
      isDemo: this.isDemo,
      runTimeoutSeconds: this.runTimeoutSeconds,
      isSubmittingTask: this.isSubmittingTask,
      submittingAction: this.submittingAction,
      uploadInfoHtml,
      currentTasksSectionHtml: this.renderHomeCurrentTasksSection(),
      recentHistorySectionHtml: this.renderHomeRecentHistorySection(),
      t,
    });
  }

  renderAlgorithmOptions(): string {
    const details = this.algorithmDetails.get(this.selectedAlgorithm);
    if (!details) {
      return `
        <div class="text-sm text-text-muted flex items-center justify-between">
          <span>${t('algorithm.loadingParams')}</span>
          <button class="btn btn-secondary text-sm" data-action="refresh-algo-list" ${this.isActionRunning('algo-refresh-list') ? 'disabled' : ''}>
            ${this.isActionRunning('algo-refresh-list') ? t('common.loading') : t('algorithm.refreshList')}
          </button>
        </div>
      `;
    }
    if (details.fields.length === 0) {
      return `
        <div class="text-sm text-text-muted flex items-center justify-between">
          <span>${t('algorithm.noAddOpt')}</span>
          <button class="btn btn-secondary text-sm" data-action="refresh-algo-list" ${this.isActionRunning('algo-refresh-list') ? 'disabled' : ''}>
            ${this.isActionRunning('algo-refresh-list') ? t('common.loading') : t('algorithm.refreshList')}
          </button>
        </div>
      `;
    }

    let html = '';

    for (const field of details.fields) {
      const fieldId = field.name.replace('add_', ''); // add_opt1 -> opt1
      const options = Object.entries(field.options);
      const label = field.text || field.name;

      if (options.length === 0) continue;

      const currentValue = fieldId === 'opt1' ? this.selectedOpt1 :
                          fieldId === 'opt2' ? this.selectedOpt2 :
                          this.selectedOpt3;

      html += `
        <div>
          <label class="block text-sm text-text-secondary mb-1">
            ${this.escapeHtml(label)} (--${this.escapeHtml(field.name)})
            ${fieldId === 'opt1' ? '<span class="text-red-500">*</span>' : ''}
          </label>
          <select class="select" id="${fieldId}-select">
            <option value="">-- Select --</option>
            ${options.map(([key, value]) => `
              <option value="${this.escapeHtml(key)}" ${currentValue === parseInt(key, 10) ? 'selected' : ''}>
                ${this.escapeHtml(key)}: ${this.escapeHtml(String(value))}
              </option>
            `).join('')}
          </select>
        </div>
      `;
    }

    return `
      ${html}
      <button class="btn btn-secondary text-sm" data-action="refresh-algo-list" ${this.isActionRunning('algo-refresh-list') ? 'disabled' : ''}>
        ${this.isActionRunning('algo-refresh-list') ? t('common.loading') : t('algorithm.refreshList')}
      </button>
    `;
  }

  renderTaskCard(task: Task): string {
    const phase = (task.phase || this.getPhaseFromStatus(task.status) || 'queueing') as NonNullable<Task['phase']>;
    const statusClass = `status-${phase === 'queueing' ? 'waiting' : phase === 'separating' ? 'processing' : phase}`;
    const statusText = phase === 'queueing'
      ? t('task.status.queueing')
      : phase === 'separating'
        ? t('task.status.separating')
        : phase === 'downloading'
          ? t('task.status.downloading')
          : t(`task.status.${task.status}`);
    const isExpanded = this.expandedTaskHashes.has(task.hash);
    const formatName = this.formats.find(f => f.id === task.format)?.name || `Format ${task.format}`;
    const selectedDownloadIndex = this.selectedDownloadFileIndexByHash.get(task.hash) ?? null;
    const isCancellingDownload = this.cancellingDownloadHashes.has(task.hash);
    return renderTaskCardHtml({
      task,
      phase,
      statusClass,
      statusText,
      isExpanded,
      formatName,
      selectedDownloadIndex,
      isCancellingDownload,
      queueQueuedFallback: this.queueInfo?.queued,
      escapeHtml: (value) => this.escapeHtml(value),
      formatBytes: (bytes) => this.formatBytes(bytes),
      getDisplayFileName: (value) => this.getDisplayFileName(value),
      t,
    });
  }

  private renderTasksFilterBar(): string {
    const inProgressTasks = this.getInProgressTasks();
    const completedTasks = this.getCompletedTasks();
    return renderTasksFilterBarHtml({
      taskViewFilter: this.taskViewFilter,
      inProgressCount: inProgressTasks.length,
      completedCount: completedTasks.length,
      historyCount: this.taskHistory.length,
      historySearchQueryEscaped: this.escapeHtml(this.historySearchQuery),
      historySortOrder: this.historySortOrder,
      t,
    });
  }

  private renderTasksViewContent(): string {
    const inProgressTasks = this.getInProgressTasks();
    const completedTasks = this.getCompletedTasks();
    const historyRecords = this.getPaginatedHistory();
    const filteredHistory = this.getFilteredHistory();
    const totalPages = Math.ceil(filteredHistory.length / this.historyPageSize);
    return renderTasksViewContentHtml({
      taskViewFilter: this.taskViewFilter,
      inProgressTasks,
      completedTasks,
      historyRecords,
      historyPage: this.historyPage,
      totalPages,
      renderTaskCard: (task) => this.renderTaskCard(task),
      renderHistoryCard: (record) => this.renderHistoryCard(record),
      t,
    });
  }

  renderTasksPage() {
    return renderTasksPageHtml({
      filterBarHtml: this.renderTasksFilterBar(),
      viewContentHtml: this.renderTasksViewContent(),
      t,
    });
  }

  renderHistoryCard(record: TaskHistoryRecord): string {
    const statusClass = record.status === 'done' ? 'status-done' : 'status-failed';
    const statusText = record.status === 'done' ? t('task.status.done') : t('task.status.failed');
    const completedDate = record.completedAt ? new Date(record.completedAt).toLocaleString() : '-';
    return renderHistoryCardHtml({
      record,
      statusClass,
      statusText,
      completedDate,
      escapeHtml: (value) => this.escapeHtml(value),
      t,
    });
  }

  renderAlgorithmsPage() {
    return renderAlgorithmsPageHtml({
      groups: this.algorithmSearchResults,
      algorithmSearchQueryEscaped: this.escapeHtml(this.algorithmSearchQuery),
      presets: this.presets,
      presetNameInputEscaped: this.escapeHtml(this.presetNameInput),
      expandedAlgorithmId: this.expandedAlgorithmId,
      renderAlgorithmDetailsSection: (algorithmId) => this.renderAlgorithmDetailsSection(algorithmId),
      escapeHtml: (text) => this.escapeHtml(text),
      isFetchingLatest: this.isActionRunning('algo-fetch-latest'),
      isRefreshingList: this.isActionRunning('algo-refresh-list'),
      t,
    });
  }

  renderAlgorithmDetailsSection(algorithmId: number): string {
    return renderAlgorithmDetailsSectionHtml({
      details: this.algorithmDetails.get(algorithmId),
      escapeHtml: (text) => this.escapeHtml(text),
      t,
    });
  }

  renderSettingsPage() {
    return renderSettingsPageHtml({
      config: this.config,
      formats: this.formats,
      connectionStatus: this.connectionStatus,
      connectionStatusText: this.connectionStatusText,
      algorithmCachePath: this.algorithmCachePath,
      isTokenVisible: this.isTokenVisible,
      isTestingConnection: this.isActionRunning('settings-test-connection'),
      isChoosingOutputDir: this.isActionRunning('settings-choose-output'),
      isSavingSettings: this.isActionRunning('settings-save'),
      t,
    });
  }

  renderAppearancePage() {
    const themes = getThemeList();
    const currentThemeId = document.documentElement.getAttribute('data-theme');
    const draft = this.customThemeDraft || {
      primary: '#0891B2',
      bgPrimary: '#ECFEFF',
      textPrimary: '#164E63',
    };
    
    return renderAppearancePageHtml({
      themes,
      currentThemeId,
      draft,
      t,
    });
  }

  renderLogsPage() {
    return renderLogsPageHtml({
      currentLogType: this.currentLogType,
      logFilterLevel: this.logFilterLevel,
      logSearchQuery: this.logSearchQuery,
      frontendLogs: this.frontendLogs,
      backendLogs: this.backendLogs,
      escapeHtml: (text: string) => this.escapeHtml(text),
      t,
    });
  }

  escapeHtml(text: string): string {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
  }

  getFilteredLogs() {
    const logs = this.currentLogType === 'frontend' ? this.frontendLogs : this.backendLogs;
    return filterLogs(logs, this.logFilterLevel, this.logSearchQuery);
  }

  async switchLogType(type: 'frontend' | 'backend') {
    this.currentLogType = type;
    if (type === 'backend') {
      await this.loadBackendLogs();
    }
    this.render();
  }

  clearLogs() {
    if (this.currentLogType === 'frontend') {
      this.frontendLogs = [];
    } else {
      this.backendLogs = [];
    }
    this.render();
  }

  exportLogs() {
    const logs = this.getFilteredLogs();
    const content = logs.map(log => `[${log.timestamp}] [${log.level}] ${log.message}`).join('\n');
    const blob = new Blob([content], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `mvsep-${this.currentLogType}-logs-${new Date().toISOString().slice(0,10)}.log`;
    a.click();
    URL.revokeObjectURL(url);
  }

  copyLogs() {
    const logs = this.getFilteredLogs();
    const content = logs.map(log => `[${log.timestamp}] [${log.level}] ${log.message}`).join('\n');
    navigator.clipboard.writeText(content).then(() => {
      this.addFrontendLog('INFO', 'Logs copied to clipboard');
      this.render();
    });
  }

  renderAboutPage() {
    return renderAboutPageHtml(t);
  }
}

const app = new App();
void app;
