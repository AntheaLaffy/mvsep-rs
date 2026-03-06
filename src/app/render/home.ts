import type { Task, TaskHistoryRecord } from '../types';

type TranslateFn = (key: string) => string;

interface RenderHomeCurrentTasksSectionArgs {
  currentTasks: Task[];
  renderTaskCard: (task: Task) => string;
  t: TranslateFn;
}

export function renderHomeCurrentTasksSectionHtml(args: RenderHomeCurrentTasksSectionArgs): string {
  if (args.currentTasks.length === 0) return '';
  return `
      <div class="card">
        <h3 class="font-semibold text-text-primary mb-4">${args.t('task.current')}</h3>
        <div class="space-y-3" id="home-current-task-list">
          ${args.currentTasks.map(task => args.renderTaskCard(task)).join('')}
        </div>
      </div>
    `;
}

interface RenderHomeRecentHistorySectionArgs {
  recentHistory: TaskHistoryRecord[];
  renderHistoryCard: (record: TaskHistoryRecord) => string;
  t: TranslateFn;
}

export function renderHomeRecentHistorySectionHtml(args: RenderHomeRecentHistorySectionArgs): string {
  if (args.recentHistory.length === 0) return '';
  return `
      <div class="card">
        <h3 class="font-semibold text-text-primary mb-4">${args.t('task.history')}</h3>
        <div class="space-y-3">
          ${args.recentHistory.map(record => args.renderHistoryCard(record)).join('')}
        </div>
      </div>
    `;
}
