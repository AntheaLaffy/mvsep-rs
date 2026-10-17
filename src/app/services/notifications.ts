import { t } from '../../i18n';
import { backendGateway } from '../backend/gateway';
import type { TaskServiceContext } from '../contracts/app-context';
import type { Task } from '../types';

// A task can reach a terminal status more than once when completion persistence
// fails and polling retries, so remember what was already announced.
const notifiedTaskHashes = new Set<string>();

function summarizeError(error: string | null | undefined): string {
  const text = (error || '').replace(/\s+/g, ' ').trim();
  if (!text) return '';
  return text.length > 160 ? `${text.slice(0, 157)}...` : text;
}

function areTaskNotificationsEnabled(app: TaskServiceContext): boolean {
  return app.config?.notify_on_task_complete === true;
}

export async function sendTestNotification(): Promise<boolean> {
  try {
    return await backendGateway.sendNotification({
      title: t('notification.testTitle'),
      body: t('notification.testBody'),
    });
  } catch (e) {
    console.error('Failed to send test notification:', e);
    return false;
  }
}

async function deliverTaskNotification(app: TaskServiceContext, task: Task): Promise<void> {
  try {
    // The in-app task list already shows the final state while the window is
    // focused, so only notify when the user is likely looking elsewhere.
    // If focus detection fails, prefer notifying over staying silent.
    let focused = false;
    try {
      focused = await backendGateway.isMainWindowFocused();
    } catch (e) {
      void app.sendDebugLog('WARN', `window focus check failed: ${String(e)}`);
    }
    if (focused) return;

    const done = task.status === 'done';
    const detail = done ? t('notification.taskDoneBody') : t('notification.taskFailedBody');
    const error = summarizeError(task.error);
    const body = !done && error
      ? `${task.file_name}: ${detail} ${error}`
      : `${task.file_name}: ${detail}`;

    const sent = await backendGateway.sendNotification({
      title: done ? t('notification.taskDoneTitle') : t('notification.taskFailedTitle'),
      body,
    });
    if (!sent) {
      app.addFrontendLog('WARN', `Desktop notification permission not granted; skipped task ${task.hash}`);
      void app.sendDebugLog('WARN', `desktop notification skipped: permission not granted, hash=${task.hash}`);
      return;
    }
    void app.sendDebugLog('INFO', `desktop notification sent: hash=${task.hash}, status=${task.status}`);
  } catch (e) {
    console.error('Failed to send task notification:', e);
    void app.sendDebugLog('ERROR', `desktop notification failed: hash=${task.hash}, err=${String(e)}`);
  }
}

export function notifyTaskFinished(app: TaskServiceContext, task: Task): void {
  if (!app.isTerminalStatus(task.status)) return;
  if (!areTaskNotificationsEnabled(app)) return;
  if (notifiedTaskHashes.has(task.hash)) return;
  notifiedTaskHashes.add(task.hash);
  void deliverTaskNotification(app, task);
}
