import type { Config, OutputFormat } from '../types';

type TranslateFn = (key: string) => string;

interface RenderSettingsPageArgs {
  config: Config | null;
  formats: OutputFormat[];
  connectionStatus: 'idle' | 'testing' | 'success' | 'error';
  connectionStatusText: string;
  algorithmCachePath: string | null;
  isTokenVisible: boolean;
  isTestingConnection: boolean;
  isChoosingOutputDir: boolean;
  isSavingSettings: boolean;
  t: TranslateFn;
}

export function renderSettingsPageHtml(args: RenderSettingsPageArgs): string {
  const {
    config,
    formats,
    connectionStatus,
    connectionStatusText,
    algorithmCachePath,
    isTokenVisible,
    isTestingConnection,
    isChoosingOutputDir,
    isSavingSettings,
    t,
  } = args;
  return `
      <div class="space-y-6 max-w-2xl">
        <div class="card">
          <h3 class="font-semibold text-text-primary mb-4">🔑 ${t('settings.api')}</h3>
          <div class="space-y-4">
            <div>
              <label class="block text-sm text-text-secondary mb-1">${t('settings.token')}</label>
              <div class="flex items-center gap-2">
                <input type="${isTokenVisible ? 'text' : 'password'}" class="input" id="token-input" value="${config?.token || ''}" />
                <button
                  class="btn btn-secondary whitespace-nowrap"
                  type="button"
                  data-action="toggle-token-visibility"
                  aria-label="${isTokenVisible ? t('settings.hideToken') : t('settings.showToken')}"
                >
                  ${isTokenVisible ? t('settings.hideToken') : t('settings.showToken')}
                </button>
              </div>
              <div class="mt-2 flex items-center gap-2 text-sm">
                <button class="text-primary underline" type="button" data-action="open-url" data-url="https://mvsep.com/user-api">
                  ${t('settings.apiKeyLink')}
                </button>
                <button
                  class="btn btn-secondary px-2 py-1 text-xs"
                  type="button"
                  data-action="show-api-help"
                  aria-label="${t('settings.apiKeyHelpAria')}"
                >
                  ?
                </button>
              </div>
            </div>
            <div>
              <label class="block text-sm text-text-secondary mb-1">${t('settings.server')}</label>
              <input type="text" class="input" id="api-url-input" value="${config?.api_url || 'https://mvsep.com'}" />
            </div>
            <div>
              <label class="block text-sm text-text-secondary mb-1">${t('settings.mirror')}</label>
              <select class="select" id="mirror-select">
                <option value="main" ${config?.mirror === 'main' ? 'selected' : ''}>Main (mvsep.com)</option>
                <option value="mirror" ${config?.mirror === 'mirror' ? 'selected' : ''}>Mirror</option>
              </select>
            </div>
            <div class="flex items-center gap-2">
              <button class="btn btn-secondary" data-action="test-connection" ${isTestingConnection || connectionStatus === 'testing' ? 'disabled' : ''}>
                ${isTestingConnection || connectionStatus === 'testing' ? t('settings.testing') : t('settings.testConnection')}
              </button>
              <span class="text-sm px-2 py-1 rounded border ${
                connectionStatus === 'success'
                  ? 'bg-green-100 text-green-700 border-green-200'
                  : connectionStatus === 'error'
                    ? 'bg-red-100 text-red-700 border-red-200'
                    : connectionStatus === 'testing'
                      ? 'bg-blue-100 text-blue-700 border-blue-200'
                      : 'bg-bg-primary text-text-muted border-border'
              }">
                ${connectionStatusText || t('settings.notTested')}
              </span>
            </div>
          </div>
        </div>

        <div class="card">
          <h3 class="font-semibold text-text-primary mb-4">📁 ${t('settings.output')}</h3>
          <div class="space-y-4">
            <div>
              <label class="block text-sm text-text-secondary mb-1">${t('settings.outputDir')}</label>
              <div class="flex gap-2">
                <input type="text" class="input" id="output-dir-input" value="${config?.output_dir || './output'}" />
                <button class="btn btn-secondary whitespace-nowrap" data-action="choose-output-dir" ${isChoosingOutputDir ? 'disabled' : ''}>
                  ${isChoosingOutputDir ? t('common.loading') : t('settings.chooseFolder')}
                </button>
              </div>
            </div>
            <div>
              <label class="block text-sm text-text-secondary mb-1">${t('settings.interval')} (${t('common.seconds')})</label>
              <input type="number" class="input" id="poll-interval-input" value="${config?.poll_interval || 5}" />
            </div>
            <div>
              <label class="block text-sm text-text-secondary mb-1">${t('settings.algorithmAutoRefreshDays')} (${t('settings.daysUnit')})</label>
              <input
                type="number"
                min="1"
                class="input"
                id="algo-auto-refresh-days-input"
                value="${config?.algorithm_auto_refresh_days || 15}"
              />
            </div>
            <div>
              <label class="block text-sm text-text-secondary mb-1">${t('settings.outputFormat')}</label>
              <select class="select" id="settings-output-format-select">
                ${formats.map(f => `
                  <option value="${f.id}" ${f.id === (config?.output_format ?? 1) ? 'selected' : ''}>${f.name}</option>
                `).join('')}
              </select>
            </div>
          </div>
        </div>

        <div class="card">
          <h3 class="font-semibold text-text-primary mb-4">🌐 ${t('settings.proxy')}</h3>
          <div class="space-y-4">
            <div>
              <label class="block text-sm text-text-secondary mb-1">${t('settings.proxyMode')}</label>
              <select class="select" id="proxy-mode-select">
                <option value="system" ${config?.proxy_mode === 'system' ? 'selected' : ''}>${t('settings.system')}</option>
                <option value="manual" ${config?.proxy_mode === 'manual' ? 'selected' : ''}>${t('settings.manual')}</option>
                <option value="none" ${config?.proxy_mode === 'none' ? 'selected' : ''}>${t('settings.none')}</option>
              </select>
            </div>
            <div class="grid grid-cols-2 gap-4">
              <div>
                <label class="block text-sm text-text-secondary mb-1">${t('settings.host')}</label>
                <input type="text" class="input" id="proxy-host-input" value="${config?.proxy_host || '127.0.0.1'}" />
              </div>
              <div>
                <label class="block text-sm text-text-secondary mb-1">${t('settings.port')}</label>
                <input type="text" class="input" id="proxy-port-input" value="${config?.proxy_port || '7897'}" />
              </div>
            </div>
          </div>
        </div>

        <div class="card">
          <h3 class="font-semibold text-text-primary mb-4">🗂️ ${t('settings.cache')}</h3>
          <div>
            <label class="block text-sm text-text-secondary mb-1">${t('settings.algorithmCachePath')}</label>
            <input
              type="text"
              class="input"
              readonly
              value="${algorithmCachePath || '-'}"
            />
          </div>
        </div>

        <button class="btn btn-primary w-full" data-action="save-settings" ${isSavingSettings ? 'disabled' : ''}>
          ${isSavingSettings ? t('common.loading') : t('settings.save')}
        </button>
      </div>
    `;
}
