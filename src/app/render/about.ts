export function renderAboutPageHtml(t: (key: string) => string): string {
  return `
      <div class="space-y-6">
        <div class="card text-center py-8">
          <div class="text-6xl mb-4">🎵</div>
          <h2 class="text-3xl font-bold text-primary">MVSEP</h2>
          <p class="text-text-secondary mb-4">${t('app.title')}</p>
          <p class="text-text-muted">Version: 1.0.0</p>
          <p class="text-text-muted">Author: 如月风铃</p>
          <p class="text-text-muted">Protocol: APLv2</p>
        </div>
        
        <div class="card">
          <h3 class="font-semibold text-text-primary mb-4">📖 ${t('about.docs')}</h3>
          <ul class="space-y-2 text-text-secondary">
            <li><button class="text-primary underline" data-action="open-url" data-url="https://github.com/AntheaLaffy/mvsep-rs">GitHub: github.com/AntheaLaffy/mvsep-rs</button></li>
            <li><button class="text-primary underline" data-action="open-url" data-url="https://mvsep.com">API: mvsep.com</button></li>
          </ul>
        </div>
        
        <div class="card">
          <h3 class="font-semibold text-text-primary mb-4">ℹ️ ${t('about.info')}</h3>
          <p class="text-text-secondary">
            MVSEP CLI - Music vocal/instrumental separation tool.<br/>
            Uses MVSEP API for audio separation.<br/>
            Supports multiple algorithms and model options.
          </p>
        </div>
        
        <div class="text-center text-text-muted">APLv2</div>
      </div>
    `;
}
