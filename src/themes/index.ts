export interface ThemeColors {
  primary: string;
  primaryLight: string;
  primaryLighter: string;
  secondary: string;
  cta: string;
  bgPrimary: string;
  bgSecondary: string;
  bgTertiary: string;
  textPrimary: string;
  textSecondary: string;
  textMuted: string;
  border: string;
}

export interface ThemeFonts {
  heading: string;
  body: string;
}

export interface Theme {
  id: string;
  name: string;
  colors: ThemeColors;
  fonts: ThemeFonts;
}

export const themes: Record<string, Theme> = {
  'fresh-cyan': {
    id: 'fresh-cyan',
    name: 'Fresh Cyan',
    colors: {
      primary: '#0891B2',
      primaryLight: '#22D3EE',
      primaryLighter: '#A5F3FC',
      secondary: '#0E7490',
      cta: '#06B6D4',
      bgPrimary: '#ECFEFF',
      bgSecondary: '#CFFAFE',
      bgTertiary: '#A5F3FC',
      textPrimary: '#164E63',
      textSecondary: '#155E75',
      textMuted: '#0E7490',
      border: '#A5F3FC',
    },
    fonts: {
      heading: 'Nunito',
      body: 'Poppins',
    },
  },
  'kawaii-pink': {
    id: 'kawaii-pink',
    name: 'Kawaii Pink',
    colors: {
      primary: '#F472B6',
      primaryLight: '#F9A8D4',
      primaryLighter: '#FCE7F3',
      secondary: '#DB2777',
      cta: '#EC4899',
      bgPrimary: '#FDF2F8',
      bgSecondary: '#FCE7F3',
      bgTertiary: '#FBCFE8',
      textPrimary: '#831843',
      textSecondary: '#9D174D',
      textMuted: '#BE185D',
      border: '#FBCFE8',
    },
    fonts: {
      heading: 'Nunito',
      body: 'Poppins',
    },
  },
  'elegant-purple': {
    id: 'elegant-purple',
    name: 'Elegant Purple',
    colors: {
      primary: '#7C3AED',
      primaryLight: '#A78BFA',
      primaryLighter: '#EDE9FE',
      secondary: '#6D28D9',
      cta: '#8B5CF6',
      bgPrimary: '#FAF5FF',
      bgSecondary: '#F3E8FF',
      bgTertiary: '#EDE9FE',
      textPrimary: '#5B21B6',
      textSecondary: '#6D28D9',
      textMuted: '#7C3AED',
      border: '#DDD6FE',
    },
    fonts: {
      heading: 'Nunito',
      body: 'Poppins',
    },
  },
  'dark': {
    id: 'dark',
    name: 'Dark Mode',
    colors: {
      primary: '#22D3EE',
      primaryLight: '#67E8F9',
      primaryLighter: '#164E63',
      secondary: '#0891B2',
      cta: '#06B6D4',
      bgPrimary: '#0F172A',
      bgSecondary: '#1E293B',
      bgTertiary: '#334155',
      textPrimary: '#F1F5F9',
      textSecondary: '#CBD5E1',
      textMuted: '#94A3B8',
      border: '#334155',
    },
    fonts: {
      heading: 'Nunito',
      body: 'Poppins',
    },
  },
};

export function applyTheme(theme: Theme): void {
  const root = document.documentElement;
  const colorVars = theme.colors;
  
  Object.entries(colorVars).forEach(([key, value]) => {
    const cssVar = `--color-${key.replace(/([A-Z])/g, '-$1').toLowerCase()}`;
    root.style.setProperty(cssVar, value);
  });

  root.style.setProperty('--font-heading', theme.fonts.heading);
  root.style.setProperty('--font-body', theme.fonts.body);
  
  root.setAttribute('data-theme', theme.id);
  localStorage.setItem('theme', theme.id);
}

export function loadTheme(): Theme {
  const savedTheme = localStorage.getItem('theme');
  if (savedTheme && themes[savedTheme]) {
    applyTheme(themes[savedTheme]);
    return themes[savedTheme];
  }
  applyTheme(themes['fresh-cyan']);
  return themes['fresh-cyan'];
}

export function getThemeList(): Theme[] {
  return Object.values(themes);
}
