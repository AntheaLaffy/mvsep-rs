export interface Config {
  token: string | null;
  api_url: string | null;
  mirror: string | null;
  proxy_mode: string | null;
  proxy_host: string | null;
  proxy_port: string | null;
  output_dir: string | null;
  output_format: number | null;
  poll_interval: number | null;
  algorithm_auto_refresh_days: number | null;
}

export interface Task {
  hash: string;
  file_name: string;
  algorithm_id: number;
  algorithm_name: string;
  model_id: number | null;
  model_name: string | null;
  model2_id?: number | null;
  model2_name?: string | null;
  model3_id?: number | null;
  model3_name?: string | null;
  format: number;
  status: string;
  progress: number;
  created_at: number;
  output_files: string[];
  error: string | null;
  message?: string | null;
  queue_count?: number | null;
  current_order?: number | null;
  phase?: 'queueing' | 'separating' | 'downloading' | 'done' | 'failed';
  download_file_name?: string | null;
  download_bytes?: number;
  download_total_bytes?: number | null;
  download_speed_bps?: number;
  download_percent?: number;
}

export interface TaskHistoryRecord {
  id: string;
  fileName: string;
  algorithmId: number;
  algorithmName: string;
  modelId: number | null;
  modelName: string | null;
  model2Id?: number | null;
  model2Name?: string | null;
  model3Id?: number | null;
  model3Name?: string | null;
  formatId: number;
  formatName: string;
  status: 'done' | 'failed';
  createdAt: number;
  completedAt: number | null;
  outputFiles: string[];
  outputPath: string | null;
  error: string | null;
}

export interface OutputFormat {
  id: number;
  name: string;
}

export interface Algorithm {
  id: number;
  name: string;
  group_id?: number;
}

export interface AlgorithmGroup {
  id: number;
  name: string;
  algorithms: Algorithm[];
}

export interface AlgorithmField {
  name: string;
  text: string;
  options: Record<string, string>;
}

export interface AlgorithmDetails {
  id: number;
  name: string;
  fields: AlgorithmField[];
}

export interface LocalAlgorithmListResponse {
  updated_at: string;
  groups: AlgorithmGroup[];
  total_algorithms: number;
}

export interface FetchLatestAlgorithmInfoResult {
  updated_at: string;
  total_groups: number;
  total_algorithms: number;
  cli_exit_code: number;
}

export interface Preset {
  id: string;
  name: string;
  algorithmId: number;
  opt1: number | null;
  opt2: number | null;
  opt3: number | null;
  formatId: number;
  demo: boolean;
}

export interface DownloadProgressPayload {
  hash: string;
  file_name: string;
  downloaded_bytes: number;
  total_bytes: number | null;
  speed_bps: number;
  percent: number;
  done: boolean;
}

export interface UploadProgressPayload {
  file_name: string;
  uploaded_bytes: number;
  total_bytes: number;
  speed_bps: number;
  percent: number;
  done: boolean;
  failed: boolean;
}

export interface TaskStatusPayload {
  status: string;
  progress: number;
  message?: string | null;
  files?: string[] | null;
  queue_count?: number | null;
  current_order?: number | null;
}

export type FrontendDebugLogArgs = { level: 'INFO' | 'WARN' | 'ERROR' | 'DEBUG'; message: string };
export type SaveConfigArgs = { config: Config };
