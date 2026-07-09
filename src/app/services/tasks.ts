import { backendGateway } from '../backend/gateway';
import type { TaskStatusPayload } from '../types';
import type { TaskServiceContext } from '../contracts/app-context';

function isDownloadCancelledError(error: unknown): boolean {
  if (!error) return false;
  const text = String(error).toLowerCase();
  return text.includes('download cancelled') || text.includes('canceled') || text.includes('cancelled');
}

function scheduleNextPoll(app: TaskServiceContext, hash: string): void {
  if (!app.taskPollingIntervals.has(hash)) return;
  const timeout = window.setTimeout(async () => {
    if (!app.taskPollingIntervals.has(hash)) return;
    if (app.pollInFlightHashes.has(hash)) {
      scheduleNextPoll(app, hash);
      return;
    }
    await pollTaskStatus(app, hash);
    if (app.taskPollingIntervals.has(hash)) {
      scheduleNextPoll(app, hash);
    }
  }, app.getPollIntervalSeconds() * 1000);
  app.taskPollingIntervals.set(hash, timeout);
}

export function startPolling(app: TaskServiceContext, hash: string): void {
  if (app.taskPollingIntervals.has(hash)) return;
  app.taskPollingIntervals.set(hash, -1);
  scheduleNextPoll(app, hash);
}

export async function pollTaskStatus(app: TaskServiceContext, hash: string): Promise<void> {
  if (!app.config) return;
  if (app.pollInFlightHashes.has(hash)) return;
  app.pollInFlightHashes.add(hash);

  try {
    const status: TaskStatusPayload = await backendGateway.getTaskStatus({
      hash,
      apiUrl: app.config.api_url,
      token: app.config.token,
    });

    const task = app.tasks.find((item) => item.hash === hash);
    if (task) {
      const normalizedStatus = app.normalizeTaskStatus(status.status);
      task.status = normalizedStatus;
      task.progress = Number.isFinite(status.progress) ? status.progress : 0;
      task.message = status.message ?? null;
      task.queue_count = status.queue_count ?? null;
      task.current_order = status.current_order ?? null;
      if (task.phase !== 'downloading') {
        const inferred = app.getPhaseFromStatus(normalizedStatus);
        if (normalizedStatus !== 'done' && normalizedStatus !== 'failed') {
          if (task.progress > 0) {
            task.phase = 'separating';
          } else if ((task.queue_count ?? 0) > 0 || (task.current_order ?? 0) > 0) {
            task.phase = 'queueing';
          } else {
            task.phase = inferred;
          }
        } else {
          task.phase = inferred;
        }
      }
      if (status.files && status.files.length > 0) {
        task.output_files = status.files;
      }
      if (status.message && normalizedStatus === 'failed') {
        task.error = status.message;
      }
    }

    const isTerminal = app.isTerminalStatus(task?.status || status.status);
    let completionPersisted = false;
    if (isTerminal) {
      stopPolling(app, hash);
      if (task) {
        try {
          await app.addToHistory(task, app.config?.output_dir || null);
          completionPersisted = true;
        } catch (e) {
          console.error('Failed to persist task history:', e);
          task.status = 'waiting';
          task.phase = 'queueing';
          task.error = 'Completion persistence failed. Status sync will retry.';
          await app.saveActiveTasks(true);
          startPolling(app, hash);
        }
      }
    }

    if (isTerminal && completionPersisted) {
      await app.saveActiveTasks(true);
    } else if (!isTerminal) {
      void app.saveActiveTasks().catch(() => undefined);
    }
    if (app.shouldRenderTaskUpdates()) {
      app.requestTaskPanelsRefresh();
    }
  } catch (e) {
    console.error('Failed to poll task status:', e);
  } finally {
    app.pollInFlightHashes.delete(hash);
  }
}

export function stopPolling(app: TaskServiceContext, hash: string): void {
  const timer = app.taskPollingIntervals.get(hash);
  if (timer !== undefined && timer >= 0) {
    clearTimeout(timer);
    app.taskPollingIntervals.delete(hash);
  } else {
    app.taskPollingIntervals.delete(hash);
  }
  app.pollInFlightHashes.delete(hash);
}

export async function downloadTask(app: TaskServiceContext, hash: string, fileIndex: number | null = null): Promise<void> {
  if (!app.config) return;
  const actionKey = `download:${hash}`;
  if (app.isActionRunning(actionKey)) return;
  app.setActionRunning(actionKey, true);
  void app.sendDebugLog('INFO', `downloadTask start: hash=${hash}, fileIndex=${fileIndex ?? 'all'}`);
  const task = app.tasks.find((item) => item.hash === hash);
  if (app.cancellingDownloadHashes.has(hash)) {
    app.setActionRunning(actionKey, false);
    return;
  }
  if (task) {
    task.phase = 'downloading';
    task.download_percent = 0;
    task.download_bytes = 0;
    task.download_total_bytes = null;
    task.download_speed_bps = 0;
    app.requestTaskPanelsRefresh();
  }

  try {
    const files = await backendGateway.downloadResult({
      hash,
      outputDir: app.config.output_dir || './output',
      fileIndex,
      originalFileName: task?.file_name ?? null,
      apiUrl: app.config.api_url,
      token: app.config.token,
    });
    app.addFrontendLog('INFO', `Downloaded ${files.length} files for task ${hash}`);
    void app.sendDebugLog('INFO', `downloadTask success: hash=${hash}, files=${files.length}`);
    let completionPersisted = false;
    if (task) {
      task.output_files = files;
      task.status = 'done';
      task.phase = 'done';
      task.download_percent = 100;
      const firstPath = files[0] || app.config.output_dir || null;
      const outputPath = firstPath ? app.getParentPath(firstPath) : null;
      try {
        await app.addToHistory(task, outputPath);
        completionPersisted = true;
      } catch (e) {
        console.error('Failed to persist task history:', e);
        task.status = 'waiting';
        task.phase = 'queueing';
        task.error = 'Completion persistence failed. Status sync will retry.';
        await app.saveActiveTasks(true);
        startPolling(app, hash);
      }
    }
    if (task && completionPersisted) {
      await app.saveActiveTasks(true);
    }
  } catch (e) {
    console.error('Failed to download task:', e);
    void app.sendDebugLog('ERROR', `downloadTask failed: hash=${hash}, err=${String(e)}`);
    const errText = String(e).toLowerCase();
    const resumableHint = errText.includes('timed out')
      || errText.includes('connection')
      || errText.includes('network')
      || errText.includes('decode')
      || errText.includes('body');
    if (task) {
      if (isDownloadCancelledError(e)) {
        task.phase = 'done';
        task.download_file_name = null;
        task.download_speed_bps = 0;
        app.addFrontendLog('INFO', `Download cancelled for task ${hash}`);
      } else {
        task.phase = 'failed';
        task.error = resumableHint
          ? 'Download interrupted. Partial file is kept and can be resumed by clicking Download again.'
          : `Download failed: ${String(e)}`;
        if (resumableHint) {
          app.addFrontendLog('WARN', `Download interrupted for task ${hash}. Resume is available on next download attempt.`);
        }
      }
    }
  } finally {
    app.setActionRunning(actionKey, false);
    if (!task || !app.isTerminalStatus(task.status)) {
      void app.saveActiveTasks().catch(() => undefined);
    }
    app.requestTaskPanelsRefresh();
  }
}

export async function cancelDownload(app: TaskServiceContext, hash: string): Promise<void> {
  if (app.cancellingDownloadHashes.has(hash)) return;
  void app.sendDebugLog('INFO', `cancelDownload requested: hash=${hash}`);
  app.cancellingDownloadHashes.add(hash);
  app.requestTaskPanelsRefresh();
  try {
    await backendGateway.cancelDownload({ hash });
    app.addFrontendLog('INFO', `Cancel requested for download task ${hash}`);
  } catch (e) {
    console.error('Failed to cancel download:', e);
    app.addFrontendLog('ERROR', `Cancel download failed for task ${hash}: ${String(e)}`);
  } finally {
    app.cancellingDownloadHashes.delete(hash);
    app.requestTaskPanelsRefresh();
  }
}
