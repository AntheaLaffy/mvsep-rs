import type { Theme } from '../../themes';

type TranslateFn = (key: string) => string;

interface ThemeDraft {
  primary: string;
  bgPrimary: string;
  textPrimary: string;
}

interface RenderAppearancePageArgs {
  themes: Theme[];
  currentThemeId: string | null;
  draft: ThemeDraft;
  t: TranslateFn;
}

export function renderAppearancePageHtml(args: RenderAppearancePageArgs): string {
  return `
      <div class="space-y-6">
        <div class="card">
          <h3 class="font-semibold text-text-primary mb-4">🎨 ${args.t('appearance.selectTheme')}</h3>
          <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
            ${args.themes.map(theme => `
              <div class="p-4 rounded-lg border-2 cursor-pointer transition-all hover:shadow-custom ${theme.id === args.currentThemeId ? 'border-primary' : 'border-border'}"
                   data-action="select-theme" data-theme-id="${theme.id}">
                <div class="h-16 rounded-lg mb-2" style="background: ${theme.colors.bgPrimary}; border: 1px solid ${theme.colors.border}">
                  <div class="h-full flex items-center justify-center gap-1">
                    <div class="w-4 h-4 rounded-full" style="background: ${theme.colors.primary}"></div>
                    <div class="w-4 h-4 rounded-full" style="background: ${theme.colors.primaryLight}"></div>
                    <div class="w-4 h-4 rounded-full" style="background: ${theme.colors.secondary}"></div>
                  </div>
                </div>
                <p class="text-sm text-center text-text-primary">${theme.name}</p>
              </div>
            `).join('')}
            <div class="p-4 rounded-lg border-2 ${args.currentThemeId === 'custom' ? 'border-primary' : 'border-border'}">
              <div class="h-16 rounded-lg mb-2 border border-border" style="background: ${args.draft.bgPrimary}">
                <div class="h-full flex items-center justify-center gap-1">
                  <div class="w-4 h-4 rounded-full" style="background: ${args.draft.primary}"></div>
                  <div class="w-4 h-4 rounded-full" style="background: ${args.draft.textPrimary}"></div>
                </div>
              </div>
              <p class="text-sm text-center text-text-primary">${args.t('appearance.customTheme')}</p>
            </div>
          </div>
        </div>

        <div class="card">
          <h3 class="font-semibold text-text-primary mb-4">🔧 ${args.t('appearance.customTheme')}</h3>
          <div class="grid md:grid-cols-3 gap-4">
            <label class="text-sm text-text-secondary">
              Primary
              <input id="custom-theme-primary" type="color" class="input h-11 p-1 mt-1" value="${args.draft.primary}">
            </label>
            <label class="text-sm text-text-secondary">
              Background
              <input id="custom-theme-bg" type="color" class="input h-11 p-1 mt-1" value="${args.draft.bgPrimary}">
            </label>
            <label class="text-sm text-text-secondary">
              Text
              <input id="custom-theme-text" type="color" class="input h-11 p-1 mt-1" value="${args.draft.textPrimary}">
            </label>
          </div>
          <div class="flex gap-2 mt-4">
            <button class="btn btn-primary" data-action="save-custom-theme">${args.t('common.save')}</button>
            <button class="btn btn-secondary" data-action="reset-custom-theme">Reset</button>
          </div>
        </div>
      </div>
    `;
}
