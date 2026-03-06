import type { Task, TaskHistoryRecord } from '../types';

export type TaskViewFilter = 'inProgress' | 'completed' | 'history';
export type TranslateFn = (key: string) => string;

interface RenderTasksFilterBarArgs {
  taskViewFilter: TaskViewFilter;
  inProgressCount: number;
  completedCount: number;
  historyCount: number;
  historySearchQueryEscaped: string;
  historySortOrder: 'desc' | 'asc';
  t: TranslateFn;
}

export function renderTasksFilterBarHtml(args: RenderTasksFilterBarArgs): string {
  return `
      <div class="flex items-center justify-between mb-4 flex-wrap gap-2">
        <div class="flex gap-2">
          <button class="btn ${args.taskViewFilter === 'inProgress' ? 'btn-primary' : 'btn-secondary'}" data-action="set-task-filter" data-filter="inProgress">
            ${args.t('common.inProgress')} (${args.inProgressCount})
          </button>
          <button class="btn ${args.taskViewFilter === 'completed' ? 'btn-primary' : 'btn-secondary'}" data-action="set-task-filter" data-filter="completed">
            ${args.t('common.completed')} (${args.completedCount})
          </button>
          <button class="btn ${args.taskViewFilter === 'history' ? 'btn-primary' : 'btn-secondary'}" data-action="set-task-filter" data-filter="history">
            ${args.t('task.history')} (${args.historyCount})
          </button>
        </div>
        <div class="flex gap-2 items-center ${args.taskViewFilter === 'history' ? '' : 'hidden'}">
          <input type="text" class="input w-48" id="history-search" placeholder="${args.t('algorithm.searchPlaceholder') || 'Search...'}" value="${args.historySearchQueryEscaped}">
          <select class="select w-28" id="history-sort">
            <option value="desc" ${args.historySortOrder === 'desc' ? 'selected' : ''}>${args.t('task.newestFirst') || 'Newest'}</option>
            <option value="asc" ${args.historySortOrder === 'asc' ? 'selected' : ''}>${args.t('task.oldestFirst') || 'Oldest'}</option>
          </select>
          <button class="btn btn-secondary" data-action="clear-history">🗑️ ${args.t('task.clearHistory') || 'Clear'}</button>
        </div>
      </div>
    `;
}

interface RenderTasksViewContentArgs {
  taskViewFilter: TaskViewFilter;
  inProgressTasks: Task[];
  completedTasks: Task[];
  historyRecords: TaskHistoryRecord[];
  historyPage: number;
  totalPages: number;
  renderTaskCard: (task: Task) => string;
  renderHistoryCard: (record: TaskHistoryRecord) => string;
  t: TranslateFn;
}

export function renderTasksViewContentHtml(args: RenderTasksViewContentArgs): string {
  if (args.taskViewFilter === 'inProgress') {
    return `
        <div class="card">
          <h3 class="font-semibold text-text-primary mb-4">${args.t('common.inProgress')}</h3>
          ${args.inProgressTasks.length === 0 ? `
            <p class="text-text-muted text-center py-4">${args.t('task.emptyInProgress')}</p>
          ` : `
            <div class="space-y-3" id="tasks-current-task-list">
              ${args.inProgressTasks.map(task => args.renderTaskCard(task)).join('')}
            </div>
          `}
        </div>
      `;
  }

  if (args.taskViewFilter === 'completed') {
    return `
        <div class="card">
          <h3 class="font-semibold text-text-primary mb-4">${args.t('common.completed')}</h3>
          ${args.completedTasks.length === 0 ? `
            <p class="text-text-muted text-center py-4">${args.t('task.emptyCompleted')}</p>
          ` : `
            <div class="space-y-3" id="tasks-current-task-list">
              ${args.completedTasks.map(task => args.renderTaskCard(task)).join('')}
            </div>
          `}
        </div>
      `;
  }

  return `
      <div class="card">
        <h3 class="font-semibold text-text-primary mb-4">${args.t('task.history')}</h3>
        ${args.historyRecords.length === 0 ? `
          <p class="text-text-muted text-center py-4">${args.t('task.noHistory') || 'No history yet'}</p>
        ` : `
          <div class="space-y-3">
            ${args.historyRecords.map(record => args.renderHistoryCard(record)).join('')}
          </div>
          ${args.totalPages > 1 ? `
            <div class="flex items-center justify-center gap-2 mt-4">
              <button class="btn btn-secondary" data-action="history-prev" ${args.historyPage === 0 ? 'disabled' : ''}>
                ← Previous
              </button>
              <span class="text-text-muted">${args.historyPage + 1} / ${args.totalPages}</span>
              <button class="btn btn-secondary" data-action="history-next" ${args.historyPage >= args.totalPages - 1 ? 'disabled' : ''}>
                Next →
              </button>
            </div>
          ` : ''}
        `}
      </div>
    `;
}

interface RenderTasksPageArgs {
  filterBarHtml: string;
  viewContentHtml: string;
  t: TranslateFn;
}

export function renderTasksPageHtml(args: RenderTasksPageArgs): string {
  return `
      <div class="space-y-4">
        <div class="card">
          <p class="text-sm text-text-muted mb-3">
            ${args.t('task.centerHint')}
          </p>
          <div id="tasks-filter-bar">${args.filterBarHtml}</div>
        </div>
        <div id="tasks-view-content">
          ${args.viewContentHtml}
        </div>
      </div>
    `;
}
