import type { Config, Preset, Task, TaskStatusPayload } from '../types';

export interface DomEventsContext {
  selectedDownloadFileIndexByHash: Map<string, number | null>;
  expandedTaskHashes: Set<string>;
  taskViewFilter: 'inProgress' | 'completed' | 'history';
  historyPage: number;
  historyPageSize: number;
  expandedAlgorithmId: number | null;
  selectedAlgorithm: number;
  selectedOpt1: number | null;
  selectedOpt2: number | null;
  selectedOpt3: number | null;
  selectedFormat: number;
  isDemo: boolean;
  runTimeoutSeconds: number;
  presetNameInput: string;
  presets: Preset[];
  selectedHomePresetId: string;
  logSearchQuery: string;
  logFilterLevel: string;
  historySearchQuery: string;
  historySortOrder: 'desc' | 'asc';
  customThemeDraft: { primary: string; bgPrimary: string; textPrimary: string } | null;
  config: Config | null;
  isLoadingAlgorithmDetails: boolean;

  getTargetElement(event: Event): HTMLElement | null;
  navigate(page: string): void;
  selectFile(): Promise<void>;
  startSeparation(): Promise<void>;
  runSeparationWorkflow(): Promise<void>;
  initializeCustomThemeDraft(): void;
  render(): void;
  downloadTask(hash: string, fileIndex?: number | null): Promise<void>;
  cancelDownload(hash: string): Promise<void>;
  deleteTask(hash: string): void;
  testConnection(): Promise<void>;
  chooseOutputDir(): Promise<void>;
  saveSettings(): Promise<void>;
  toggleTokenVisibility(): void;
  showApiTokenHelp(): void;
  switchLogType(type: 'frontend' | 'backend'): Promise<void>;
  clearLogs(): void;
  exportLogs(): void;
  copyLogs(): void;
  confirmAction(message: string): boolean;
  clearHistory(): void;
  deleteFromHistory(id: string): void;
  retryFromHistory(id: string): Promise<void>;
  openOutput(path: string): Promise<boolean>;
  openHistoryOutput(id: string): Promise<boolean>;
  loadAlgorithmDetails(algorithmId: number): Promise<unknown>;
  refreshAlgorithmListFromLocal(options?: { render?: boolean; syncSearch?: boolean }): Promise<void>;
  fetchLatestAlgorithmInfo(): Promise<void>;
  createPresetFromCurrent(name: string): Preset;
  savePresets(): void;
  applyPresetById(presetId: string): Promise<void>;
  saveCustomTheme(): void;
  getFilteredHistory(): unknown[];
  onAlgorithmSearchInput(query: string): void;
  showTransientNotice(message: string, level?: 'info' | 'warn', durationMs?: number): void;
  sendDebugLog(level: 'INFO' | 'WARN' | 'ERROR' | 'DEBUG', message: string): Promise<void>;
  getApiUrlByMirror(mirror: string): string;
  handleFileDrop(file: File): void;
}

export interface TaskServiceContext {
  config: Config | null;
  tasks: Task[];
  pollInFlightHashes: Set<string>;
  taskPollingIntervals: Map<string, number>;
  cancellingDownloadHashes: Set<string>;

  getPollIntervalSeconds(): number;
  normalizeTaskStatus(status: string): string;
  getPhaseFromStatus(status: string): Task['phase'];
  isTerminalStatus(status: string): boolean;
  addToHistory(task: Task, outputPath?: string | null): void;
  saveActiveTasks(force?: boolean): void;
  shouldRenderTaskUpdates(): boolean;
  requestTaskPanelsRefresh(): void;
  addFrontendLog(level: string, message: string): void;
  sendDebugLog(level: 'INFO' | 'WARN' | 'ERROR' | 'DEBUG', message: string): Promise<void>;
  getParentPath(path: string): string;
}

export type TaskStatusInvoker = (hash: string, apiUrl: string | null, token: string | null) => Promise<TaskStatusPayload>;
