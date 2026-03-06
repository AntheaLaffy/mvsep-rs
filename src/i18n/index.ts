import en from './locales/en.json';
import zhCN from './locales/zh-CN.json';
import ja from './locales/ja.json';

export type Locale = 'en' | 'zh-CN' | 'ja';

export interface LocaleInfo {
  code: Locale;
  name: string;
  nativeName: string;
}

export const locales: Record<Locale, Record<string, any>> = {
  'en': en,
  'zh-CN': zhCN,
  'ja': ja,
};

export const localeInfo: Record<Locale, LocaleInfo> = {
  'en': { code: 'en', name: 'English', nativeName: 'EN' },
  'zh-CN': { code: 'zh-CN', name: 'Chinese', nativeName: '中文' },
  'ja': { code: 'ja', name: 'Japanese', nativeName: '日本語' },
};

let currentLocale: Locale = 'en';

export function setLocale(locale: Locale): void {
  currentLocale = locale;
  localStorage.setItem('locale', locale);
  document.documentElement.lang = locale;
}

export function getLocale(): Locale {
  return currentLocale;
}

export function initLocale(): Locale {
  const saved = localStorage.getItem('locale') as Locale | null;
  if (saved && locales[saved]) {
    currentLocale = saved;
    document.documentElement.lang = saved;
    return saved;
  }
  
  const browserLang = navigator.language;
  if (browserLang.startsWith('zh')) {
    currentLocale = 'zh-CN';
  } else if (browserLang.startsWith('ja')) {
    currentLocale = 'ja';
  } else {
    currentLocale = 'en';
  }
  
  document.documentElement.lang = currentLocale;
  return currentLocale;
}

export function t(key: string): string {
  const keys = key.split('.');
  let value: any = locales[currentLocale];
  
  for (const k of keys) {
    if (value && typeof value === 'object' && k in value) {
      value = value[k];
    } else {
      return key;
    }
  }
  
  return typeof value === 'string' ? value : key;
}

export function getAvailableLocales(): LocaleInfo[] {
  return Object.values(localeInfo);
}
