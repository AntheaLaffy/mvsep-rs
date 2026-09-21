type TranslateFn = (key: string) => string;

interface RenderAboutPageArgs {
  appVersion: string;
  updateCheckStatus: 'idle' | 'checking' | 'current' | 'available' | 'error';
  latestVersion: string | null;
  latestReleaseUrl: string;
  isLatestVersionIgnored: boolean;
  t: TranslateFn;
}

export function renderAboutPageHtml(args: RenderAboutPageArgs): string {
  const { appVersion, updateCheckStatus, latestVersion, latestReleaseUrl, isLatestVersionIgnored, t } = args;
  const updateMessage = updateCheckStatus === 'current'
    ? t('about.upToDate')
    : updateCheckStatus === 'available'
      ? `${t('about.updateAvailable')} v${latestVersion}`
      : updateCheckStatus === 'error'
        ? t('about.updateCheckFailed')
        : '';

  return `
      <div class="space-y-6">
        <div class="card text-center py-8">
          <div class="text-6xl mb-4">🎵</div>
          <h2 class="text-3xl font-bold text-primary">MVSEP</h2>
          <p class="text-text-secondary mb-4">${t('app.title')}</p>
          <p class="text-text-muted">${t('about.version')}: ${appVersion}</p>
          <p class="text-text-muted">Author: 如月风铃</p>
          <p class="text-text-muted">License: Apache-2.0</p>
          <div class="mt-5 flex flex-wrap items-center justify-center gap-3">
            <button class="btn btn-primary" data-action="check-for-updates" ${updateCheckStatus === 'checking' ? 'disabled' : ''}>
              ${updateCheckStatus === 'checking' ? t('about.checkingUpdates') : t('about.checkUpdates')}
            </button>
            ${updateCheckStatus === 'available' ? `
              <button class="btn btn-cta" data-action="open-url" data-url="${latestReleaseUrl}">${t('about.downloadUpdate')}</button>
              <button class="btn btn-secondary" data-action="toggle-ignore-update">
                ${isLatestVersionIgnored ? t('about.restoreUpdateNotifications') : t('about.ignoreThisVersion')}
              </button>
            ` : ''}
          </div>
          ${updateMessage ? `<p class="mt-3 text-sm ${updateCheckStatus === 'error' ? 'text-red-500' : 'text-text-secondary'}">${updateMessage}</p>` : ''}
        </div>

        <div class="card">
          <h3 class="font-semibold text-text-primary mb-4">${t('about.support')}</h3>
          <div class="flex flex-wrap gap-3">
            <button class="btn btn-secondary" data-action="open-url" data-url="https://github.com/AntheaLaffy/mvsep-rs">GitHub</button>
            <button class="btn btn-primary" data-action="open-url" data-url="https://github.com/AntheaLaffy/mvsep-rs/issues/new/choose">${t('about.feedback')}</button>
            <button class="btn btn-secondary" data-action="open-url" data-url="https://mvsep.com">MVSEP API</button>
          </div>
        </div>

        <div class="card">
          <h3 class="font-semibold text-text-primary mb-4">${t('about.info')}</h3>
          <p class="text-text-secondary">${t('about.description')}</p>
        </div>
      </div>
    `;
}
