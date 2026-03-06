import type { AlgorithmDetails, AlgorithmGroup, Preset } from '../types';

type TranslateFn = (key: string) => string;

interface RenderAlgorithmDetailsSectionArgs {
  details: AlgorithmDetails | undefined;
  escapeHtml: (text: string) => string;
  t: TranslateFn;
}

export function renderAlgorithmDetailsSectionHtml(args: RenderAlgorithmDetailsSectionArgs): string {
  const { details, escapeHtml, t } = args;
  if (!details) {
    return `<p class="text-sm text-text-muted mt-3">${t('common.loading')}</p>`;
  }
  if (details.fields.length === 0) {
    return `<p class="text-sm text-text-muted mt-3">${t('algorithm.noModelParams')}</p>`;
  }

  return `
      <div class="mt-3 p-3 rounded-lg bg-bg-secondary border border-border">
        ${details.fields.map(field => `
          <div class="mb-2 last:mb-0">
            <p class="text-sm font-semibold text-text-primary">${field.text || field.name} (--${field.name})</p>
            <p class="text-xs text-text-muted mt-1">
              ${Object.entries(field.options).map(([key, value]) => `${key}: ${escapeHtml(value)}`).join(' | ')}
            </p>
          </div>
        `).join('')}
      </div>
    `;
}

interface RenderAlgorithmsPageArgs {
  groups: AlgorithmGroup[];
  algorithmSearchQueryEscaped: string;
  presets: Preset[];
  presetNameInputEscaped: string;
  expandedAlgorithmId: number | null;
  renderAlgorithmDetailsSection: (algorithmId: number) => string;
  escapeHtml: (text: string) => string;
  t: TranslateFn;
  isFetchingLatest: boolean;
  isRefreshingList: boolean;
}

export function renderAlgorithmsPageHtml(args: RenderAlgorithmsPageArgs): string {
  return `
      <div class="space-y-4">
        <div class="card">
          <div class="flex gap-2 items-center">
            <input
              type="text"
              class="input"
              id="algorithm-search-input"
              placeholder="${args.t('algorithm.searchPlaceholder')}"
              value="${args.algorithmSearchQueryEscaped}"
            />
            <button class="btn btn-secondary whitespace-nowrap" data-action="fetch-latest-algo-info" ${args.isFetchingLatest ? 'disabled' : ''}>
              ${args.isFetchingLatest ? args.t('common.loading') : args.t('algorithm.fetchLatestInfo')}
            </button>
            <button class="btn btn-secondary whitespace-nowrap" data-action="refresh-algo-list" ${args.isRefreshingList ? 'disabled' : ''}>
              ${args.isRefreshingList ? args.t('common.loading') : args.t('algorithm.refreshList')}
            </button>
          </div>
        </div>

        <div class="card space-y-3">
          <h3 class="font-semibold text-text-primary">${args.t('algorithm.presetTitle')}</h3>
          <div class="flex gap-2">
            <input class="input" id="preset-name-input" placeholder="${args.t('algorithm.presetNamePlaceholder')}" value="${args.presetNameInputEscaped}" />
            <button class="btn btn-primary whitespace-nowrap" data-action="save-preset">${args.t('algorithm.saveCurrentConfig')}</button>
          </div>
          ${args.presets.length > 0 ? `
            <div class="space-y-2">
              ${args.presets.map(p => `
                <div class="p-3 rounded-lg bg-bg-primary border border-border flex items-center justify-between gap-2">
                  <div class="text-sm">
                    <p class="font-medium">${args.escapeHtml(p.name)}</p>
                    <p class="text-text-muted">${args.t('home.algorithm')} #${p.algorithmId} / opt1=${p.opt1 ?? '-'} opt2=${p.opt2 ?? '-'} opt3=${p.opt3 ?? '-'}</p>
                  </div>
                  <div class="flex gap-2">
                    <button class="btn btn-secondary text-sm" data-action="apply-algo-preset" data-id="${p.id}">${args.t('common.apply')}</button>
                    <button class="btn btn-secondary text-sm" data-action="delete-preset" data-id="${p.id}">${args.t('task.action.delete')}</button>
                  </div>
                </div>
              `).join('')}
            </div>
          ` : `<p class="text-sm text-text-muted">${args.t('algorithm.noPresets')}</p>`}
        </div>
        
        <div class="space-y-4">
          ${args.groups.length === 0 ? `
            <div class="card text-text-muted">${args.t('algorithm.noneFound')}</div>
          ` : args.groups.map(group => `
            <div class="card">
              <h3 class="font-semibold text-text-primary mb-3">${group.name}</h3>
              <div class="space-y-2">
                ${group.algorithms.map(algo => `
                  <div class="p-3 bg-bg-primary rounded-lg border border-border">
                    <div class="flex items-center justify-between gap-2">
                      <span class="text-text-primary">${algo.id}: ${algo.name}</span>
                      <div class="flex gap-2">
                        <button class="btn btn-secondary text-sm" data-action="toggle-algo-details" data-id="${algo.id}">
                          ${args.expandedAlgorithmId === algo.id ? args.t('common.collapse') : args.t('task.action.view')}
                        </button>
                        <button class="btn btn-primary text-sm" data-action="use-algorithm" data-id="${algo.id}">
                          ${args.t('common.use')}
                        </button>
                      </div>
                    </div>
                    ${args.expandedAlgorithmId === algo.id ? args.renderAlgorithmDetailsSection(algo.id) : ''}
                  </div>
                `).join('')}
              </div>
            </div>
          `).join('')}
        </div>
      </div>
    `;
}
