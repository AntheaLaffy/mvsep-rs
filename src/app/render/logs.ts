export interface LogEntry {
  timestamp: string;
  level: string;
  message: string;
}

export function filterLogs(logs: LogEntry[], level: string, searchQuery: string): LogEntry[] {
  let filteredLogs = logs;
  if (level !== 'all') {
    filteredLogs = filteredLogs.filter((log) => log.level === level);
  }
  if (searchQuery) {
    const query = searchQuery.toLowerCase();
    filteredLogs = filteredLogs.filter((log) =>
      log.message.toLowerCase().includes(query) || log.timestamp.toLowerCase().includes(query),
    );
  }
  return filteredLogs;
}

interface RenderLogsPageArgs {
  currentLogType: 'frontend' | 'backend';
  logFilterLevel: string;
  logSearchQuery: string;
  frontendLogs: LogEntry[];
  backendLogs: LogEntry[];
  escapeHtml: (text: string) => string;
  t: (key: string) => string;
}

export function renderLogsPageHtml(args: RenderLogsPageArgs): string {
  const logs = args.currentLogType === 'frontend' ? args.frontendLogs : args.backendLogs;
  const filteredLogs = filterLogs(logs, args.logFilterLevel, args.logSearchQuery);

  const logEntriesHtml = filteredLogs
    .map((log) => {
      const levelClass = log.level === 'ERROR'
        ? 'text-red-500'
        : log.level === 'WARN'
          ? 'text-yellow-500'
          : log.level === 'DEBUG'
            ? 'text-gray-500'
            : 'text-blue-500';
      return `<div class="py-1 border-b border-border last:border-0">
        <span class="text-text-muted">[${log.timestamp}]</span>
        <span class="${levelClass} ml-2">[${log.level}]</span>
        <span class="text-text-primary ml-2">${args.escapeHtml(log.message)}</span>
      </div>`;
    })
    .join('');

  return `
      <div class="space-y-4">
        <div class="card">
          <div class="flex items-center justify-between mb-4 flex-wrap gap-2">
            <div class="flex gap-2">
              <button class="btn ${args.currentLogType === 'frontend' ? 'btn-primary' : 'btn-secondary'}" data-action="switch-log-type" data-type="frontend">
                ${args.t('logs.frontend')}
              </button>
              <button class="btn ${args.currentLogType === 'backend' ? 'btn-primary' : 'btn-secondary'}" data-action="switch-log-type" data-type="backend">
                ${args.t('logs.backend')}
              </button>
            </div>
            <div class="flex gap-2 items-center">
              <input type="text" class="input w-48" id="log-search" placeholder="${args.t('logs.search') || 'Search...'}" value="${args.escapeHtml(args.logSearchQuery)}">
              <select class="select w-28" id="log-level-filter">
                <option value="all" ${args.logFilterLevel === 'all' ? 'selected' : ''}>${args.t('logs.all') || 'All'}</option>
                <option value="INFO" ${args.logFilterLevel === 'INFO' ? 'selected' : ''}>INFO</option>
                <option value="WARN" ${args.logFilterLevel === 'WARN' ? 'selected' : ''}>WARN</option>
                <option value="ERROR" ${args.logFilterLevel === 'ERROR' ? 'selected' : ''}>ERROR</option>
                <option value="DEBUG" ${args.logFilterLevel === 'DEBUG' ? 'selected' : ''}>DEBUG</option>
              </select>
              <button class="btn btn-secondary" data-action="clear-logs">🗑️ ${args.t('logs.clear')}</button>
              <button class="btn btn-secondary" data-action="export-logs">📥 ${args.t('logs.export')}</button>
              <button class="btn btn-secondary" data-action="copy-logs">📋 ${args.t('logs.copy')}</button>
            </div>
          </div>
          <div class="bg-bg-primary rounded-lg p-4 h-[500px] overflow-auto font-mono text-sm" id="log-container">
            ${logEntriesHtml || '<p class="text-text-muted">No logs available</p>'}
          </div>
          <div class="mt-2 text-sm text-text-muted">
            Showing ${filteredLogs.length} of ${logs.length} entries
          </div>
        </div>
      </div>
    `;
}
