import type { Task, TaskHistoryRecord } from '../types';

type TranslateFn = (key: string) => string;

interface RenderTaskCardArgs {
  task: Task;
  phase: NonNullable<Task['phase']>;
  statusClass: string;
  statusText: string;
  isExpanded: boolean;
  formatName: string;
  selectedDownloadIndex: number | null;
  isCancellingDownload: boolean;
  queueQueuedFallback: number | null | undefined;
  escapeHtml: (value: string) => string;
  formatBytes: (bytes: number | null | undefined) => string;
  getDisplayFileName: (pathOrName: string) => string;
  t: TranslateFn;
}

export function renderTaskCardHtml(args: RenderTaskCardArgs): string {
  const { task } = args;
  return `
      <div class="p-4 bg-bg-primary rounded-lg border border-border" data-task-hash="${task.hash}">
        <div class="flex items-center justify-between mb-2">
          <span class="font-medium">🎧 ${args.escapeHtml(task.file_name)}</span>
          <span class="status-badge ${args.statusClass}">${args.statusText}</span>
        </div>
        ${args.phase === 'separating' ? `
          <div class="progress-bar mb-2">
            <div class="progress-bar-fill" style="width: ${task.progress}%"></div>
          </div>
          <p class="text-sm text-text-muted">${task.progress.toFixed(0)}%</p>
        ` : ''}
        ${args.phase === 'queueing' ? `
          <p class="text-sm text-text-muted mb-2">
            ${args.t('task.queueCountLabel')}: ${task.queue_count ?? args.queueQueuedFallback ?? '-'}，${args.t('task.queueOrderLabel')}: ${task.current_order ?? '-'}
          </p>
        ` : ''}
        ${args.phase === 'downloading' ? `
          <div class="progress-bar mb-2">
            <div class="progress-bar-fill" style="width: ${task.download_percent ?? 0}%"></div>
          </div>
          <p class="text-sm text-text-muted mb-1">
            ${task.download_file_name ? `${args.t('task.fileLabel')}: ${args.escapeHtml(task.download_file_name)} / ` : ''}${(task.download_percent ?? 0).toFixed(1)}%
          </p>
          <p class="text-sm text-text-muted">
            ${args.formatBytes(task.download_bytes)} / ${args.formatBytes(task.download_total_bytes)} · ${args.formatBytes(task.download_speed_bps)}/s
          </p>
        ` : ''}
        ${task.status === 'done' ? `
          <p class="text-sm text-text-muted mb-2">${task.output_files.length} files</p>
          ${task.output_files.length > 0 ? `
            <div class="mb-2">
              <label class="block text-xs text-text-muted mb-1">${args.t('task.downloadTarget')}</label>
              <select class="select text-sm" data-action="select-download-target" data-hash="${task.hash}">
                <option value="-1" ${args.selectedDownloadIndex === null ? 'selected' : ''}>${args.t('task.downloadAll')}</option>
                ${task.output_files.map((f, idx) => `
                  <option value="${idx}" ${args.selectedDownloadIndex === idx ? 'selected' : ''}>${idx + 1}. ${args.escapeHtml(args.getDisplayFileName(f))}</option>
                `).join('')}
              </select>
            </div>
          ` : ''}
        ` : ''}
        ${task.status === 'failed' ? `
          <p class="text-sm text-red-500 mb-2">${args.escapeHtml(task.error || 'Failed')}</p>
        ` : ''}
        ${task.message ? `<p class="text-sm text-text-muted mb-2">${args.escapeHtml(task.message)}</p>` : ''}
        ${args.isExpanded ? `
          <div class="mt-2 p-3 rounded-lg bg-bg-secondary border border-border text-sm text-text-secondary">
            <p>Hash: ${args.escapeHtml(task.hash)}</p>
            <p>Algorithm: ${args.escapeHtml(task.algorithm_name)} (#${task.algorithm_id})</p>
            <p>Model: ${args.escapeHtml(task.model_name || '-')}</p>
            <p>Format: ${args.escapeHtml(args.formatName)}</p>
            <p>Created: ${new Date(task.created_at).toLocaleString()}</p>
          </div>
        ` : ''}
        <div class="flex gap-2 mt-2">
          <button class="btn btn-secondary text-sm" data-action="toggle-task-details">
            ${args.isExpanded ? args.t('common.close') : args.t('task.action.view')}
          </button>
          ${task.status === 'done' ? `
            <button class="btn btn-primary text-sm" data-action="download-task">
              📥 ${args.t('task.action.download')}
            </button>
          ` : ''}
          ${args.phase === 'downloading' ? `
            <button class="btn btn-secondary text-sm" data-action="cancel-download" ${args.isCancellingDownload ? 'disabled' : ''}>
              ${args.isCancellingDownload ? `${args.t('common.loading')}...` : args.t('task.action.cancel')}
            </button>
          ` : ''}
          ${((task.status === 'done' || task.status === 'failed') && args.phase !== 'downloading') ? `
            <button class="btn btn-secondary text-sm" data-action="delete-task">
              🗑️ ${args.t('task.action.delete')}
            </button>
          ` : ''}
        </div>
      </div>
    `;
}

interface RenderHistoryCardArgs {
  record: TaskHistoryRecord;
  statusClass: string;
  statusText: string;
  completedDate: string;
  escapeHtml: (value: string) => string;
  t: TranslateFn;
}

export function renderHistoryCardHtml(args: RenderHistoryCardArgs): string {
  const { record } = args;
  return `
      <div class="p-4 bg-bg-primary rounded-lg border border-border">
        <div class="flex items-center justify-between mb-2">
          <span class="font-medium">🎧 ${args.escapeHtml(record.fileName)}</span>
          <span class="status-badge ${args.statusClass}">${args.statusText}</span>
        </div>
        <div class="text-sm text-text-muted space-y-1">
          <p>📊 ${args.escapeHtml(record.algorithmName)} ${record.modelName ? `(${args.escapeHtml(record.modelName)})` : ''}</p>
          <p>📁 ${args.escapeHtml(record.formatName)}</p>
          <p>🕐 ${args.completedDate}</p>
          ${record.outputFiles.length > 0 ? `<p>📄 ${record.outputFiles.length} files</p>` : ''}
          ${record.error ? `<p class="text-red-500">❌ ${args.escapeHtml(record.error)}</p>` : ''}
        </div>
        <div class="flex gap-2 mt-3">
          ${record.status === 'done' ? `
            <button class="btn btn-secondary text-sm" data-action="open-history-output" data-id="${record.id}">
              📂 ${args.t('task.openFolder') || 'Open'}
            </button>
          ` : ''}
          <button class="btn btn-secondary text-sm" data-action="retry-task" data-id="${record.id}">
            🔄 ${args.t('task.action.retry')}
          </button>
        </div>
      </div>
    `;
}
