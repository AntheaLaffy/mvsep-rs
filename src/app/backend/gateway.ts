import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import { openPath, revealItemInDir, openUrl as openExternalUrl } from '@tauri-apps/plugin-opener';
import type {
  AlgorithmDetails,
  Config,
  DownloadProgressPayload,
  FetchLatestAlgorithmInfoResult,
  FrontendDebugLogArgs,
  LocalAlgorithmListResponse,
  OutputFormat,
  Task,
  TaskHistoryRecord,
  TaskStatusPayload,
  UploadProgressPayload,
} from '../types';

type Unlisten = () => void;

export type OpenFileDialogOptions = Parameters<typeof openDialog>[0];
export type QueueStatus = { active: number; queued: number };

type ApiAuthArgs = {
  apiUrl: string | null;
  token: string | null;
};

export const backendGateway = {
  loadConfig(): Promise<Config> {
    return invoke<Config>('load_config');
  },

  saveConfig(args: { config: Config }): Promise<void> {
    return invoke<void>('save_config', args);
  },

  resolvePath(args: { path: string }): Promise<string> {
    return invoke<string>('resolve_path', args);
  },

  openInFileManager(args: { path: string }): Promise<void> {
    return invoke<void>('open_in_file_manager', args);
  },

  openPath(path: string): Promise<void> {
    return openPath(path);
  },

  revealItemInDir(path: string): Promise<void> {
    return revealItemInDir(path);
  },

  openExternalUrl(url: string): Promise<void> {
    return openExternalUrl(url);
  },

  openFileDialog(options: OpenFileDialogOptions): Promise<string | string[] | null> {
    return openDialog(options) as Promise<string | string[] | null>;
  },

  testConnection(args: ApiAuthArgs): Promise<boolean> {
    return invoke<boolean>('test_connection', args);
  },

  fetchLatestAlgorithmInfo(args: ApiAuthArgs & {
    proxyMode?: string | null;
    proxyHost?: string | null;
    proxyPort?: string | null;
  }): Promise<FetchLatestAlgorithmInfoResult> {
    return invoke<FetchLatestAlgorithmInfoResult>('fetch_latest_algorithm_info', args);
  },

  refreshAlgorithmListFromLocal(): Promise<LocalAlgorithmListResponse> {
    return invoke<LocalAlgorithmListResponse>('refresh_algorithm_list_from_local');
  },

  getAlgorithmCachePath(): Promise<string> {
    return invoke<string>('get_algorithm_cache_path_cmd');
  },

  getAlgorithmDetailsFromLocal(args: { algorithmId: number }): Promise<AlgorithmDetails> {
    return invoke<AlgorithmDetails>('get_algorithm_details_from_local', args);
  },

  listFormats(args: ApiAuthArgs): Promise<OutputFormat[]> {
    return invoke<OutputFormat[]>('list_formats', args);
  },

  getQueueInfo(args: ApiAuthArgs): Promise<QueueStatus> {
    return invoke<QueueStatus>('get_queue_info', args);
  },

  createTask(args: {
    filePath: string;
    sepType: number;
    opt1: number | null;
    opt2: number | null;
    opt3: number | null;
    outputFormat: number;
    demo: boolean;
    apiUrl: string | null;
    token: string | null;
  }): Promise<string> {
    return invoke<string>('create_task', args);
  },

  getTaskStatus(args: ApiAuthArgs & { hash: string }): Promise<TaskStatusPayload> {
    return invoke<TaskStatusPayload>('get_task_status', args);
  },

  downloadResult(args: {
    hash: string;
    outputDir: string;
    fileIndex: number | null;
    originalFileName: string | null;
    apiUrl: string | null;
    token: string | null;
  }): Promise<string[]> {
    return invoke<string[]>('download_result', args);
  },

  cancelDownload(args: { hash: string }): Promise<void> {
    return invoke<void>('cancel_download', args);
  },

  getTasks(): Promise<Task[]> {
    return invoke<Task[]>('get_tasks');
  },

  replaceActiveTasks(args: { tasks: Task[] }): Promise<void> {
    return invoke<void>('replace_active_tasks', args);
  },

  getTaskHistory(): Promise<TaskHistoryRecord[]> {
    return invoke<TaskHistoryRecord[]>('get_task_history');
  },

  saveTaskHistory(args: { records: TaskHistoryRecord[] }): Promise<void> {
    return invoke<void>('save_task_history', args);
  },

  completeTask(args: { task: Task; record: TaskHistoryRecord }): Promise<void> {
    return invoke<void>('complete_task', args);
  },

  getBackendLogs(): Promise<Array<{ timestamp: string; level: string; message: string }>> {
    return invoke<Array<{ timestamp: string; level: string; message: string }>>('get_backend_logs');
  },

  sendFrontendDebugLog(args: FrontendDebugLogArgs): Promise<void> {
    return invoke<void>('frontend_debug_log', args);
  },

  onDownloadProgress(handler: (payload: DownloadProgressPayload) => void): Promise<Unlisten> {
    return listen<DownloadProgressPayload>('download-progress', (event) => handler(event.payload));
  },

  onUploadProgress(handler: (payload: UploadProgressPayload) => void): Promise<Unlisten> {
    return listen<UploadProgressPayload>('upload-progress', (event) => handler(event.payload));
  },
};
