type TranslateFn = (key: string) => string;

interface RenderHomePageArgs {
  isInitialLoading: boolean;
  selectedFile: string | null;
  selectedFileLabelEscaped: string;
  algorithmSelectOptionsHtml: string;
  algorithmOptionsHtml: string;
  presetBlockHtml: string;
  formatOptionsHtml: string;
  isDemo: boolean;
  runTimeoutSeconds: number;
  isSubmittingTask: boolean;
  submittingAction: 'create' | 'run' | null;
  uploadInfoHtml: string;
  currentTasksSectionHtml: string;
  recentHistorySectionHtml: string;
  t: TranslateFn;
}

export function renderHomePageHtml(args: RenderHomePageArgs): string {
  return `
      <div class="space-y-6">
        ${args.isInitialLoading ? `
          <div class="card">
            <p class="text-sm text-text-secondary">${args.t('home.initialLoadingHint')}</p>
          </div>
        ` : ''}
        <div class="card">
          <div id="dropzone" class="dropzone" data-action="select-file">
            <div class="text-4xl mb-4">🎶</div>
            <p class="text-lg text-text-primary mb-2">${args.t('home.dropzone')}</p>
            <p class="text-text-muted">${args.t('home.selectFile')}</p>
            ${args.selectedFile ? `
              <div class="mt-4 px-4 py-2 bg-bg-tertiary rounded-lg">
                📁 ${args.selectedFileLabelEscaped}
              </div>
            ` : ''}
            <button class="btn btn-primary mt-4" data-action="select-file">
              ${args.t('home.selectFile')}
            </button>
          </div>
        </div>

        ${args.selectedFile ? `
          <div class="card space-y-4">
            <h3 class="font-semibold text-text-primary">${args.t('home.algorithm')}</h3>
            <select class="select" id="algorithm-select">
              ${args.algorithmSelectOptionsHtml}
            </select>

            ${args.algorithmOptionsHtml}

            ${args.presetBlockHtml}

            <h3 class="font-semibold text-text-primary">${args.t('home.format')}</h3>
            <select class="select" id="format-select">
              ${args.formatOptionsHtml}
            </select>

            <label class="flex items-center gap-2">
              <input type="checkbox" id="demo-checkbox" ${args.isDemo ? 'checked' : ''}>
              <span>${args.t('home.publishDemo')}</span>
            </label>

            <label class="block text-sm text-text-secondary">
              Run Timeout (s)
              <input type="number" class="input mt-1" id="run-timeout-input" min="30" value="${args.runTimeoutSeconds}">
            </label>

            <p class="text-xs text-text-muted">
              ${args.t('home.runHint')}
            </p>
            ${args.isSubmittingTask ? args.uploadInfoHtml : ''}

            <button class="btn btn-cta w-full" data-action="start-separation" ${args.isSubmittingTask ? 'disabled' : ''}>
              ${args.submittingAction === 'create' ? args.t('home.createPending') : args.t('home.createTask')}
            </button>
            <button class="btn btn-primary w-full" data-action="start-run-workflow" ${args.isSubmittingTask ? 'disabled' : ''}>
              ${args.submittingAction === 'run' ? args.t('home.runPending') : args.t('home.oneClickRun')}
            </button>
          </div>
        ` : ''}
        <div id="home-current-tasks-section">${args.currentTasksSectionHtml}</div>
        <div id="home-recent-history-section">${args.recentHistorySectionHtml}</div>
      </div>
    `;
}
