import type { Locale } from './index';

export function detectLocale(language: string): Locale {
  const code = language.trim().split(/[_.@-]/)[0].toLowerCase();
  if (code === 'zh') return 'zh-CN';
  if (code === 'ja') return 'ja';
  return 'en';
}
