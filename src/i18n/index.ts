import { invoke } from '@tauri-apps/api/core';
import { logger } from '../app/utils/logger';
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

async function webStorageGet(key: string): Promise<string | null> {
  try {
    logger.info(`[STORAGE] Getting key: "${key}"`);
    const result = await invoke('web_storage_get', { key });
    logger.info(`[STORAGE] Got key "${key}": ${result !== null ? `"${result}"` : 'null'}`);
    return result as string | null;
  } catch (e) {
    logger.error(`[STORAGE] Error getting key "${key}": ${String(e)}`);
    return null;
  }
}

async function webStorageSet(key: string, value: string): Promise<void> {
  try {
    logger.info(`[STORAGE] Setting key: "${key}", value: "${value}"`);
    await invoke('web_storage_set', { key, value });
    logger.info(`[STORAGE] Successfully set key "${key}"`);
  } catch (e) {
    logger.error(`[STORAGE] Error setting key "${key}": ${String(e)}`);
  }
}

async function migrateFromLocalStorage(key: string): Promise<string | null> {
  logger.info(`[MIGRATE] Checking migration for key: "${key}"`);
  const migrated = await webStorageGet(`migrated_${key}`);
  logger.info(`[MIGRATE] Migration flag: "${migrated}"`);
  if (migrated === '1') {
    logger.info(`[MIGRATE] Already migrated, reading from web.db`);
    return await webStorageGet(key);
  }
  
  const localStorageValue = localStorage.getItem(key);
  logger.info(`[MIGRATE] localStorage value: "${localStorageValue}"`);
  if (localStorageValue) {
    logger.info(`[MIGRATE] Found localStorage value, migrating to web.db`);
    await webStorageSet(key, localStorageValue);
    await webStorageSet(`migrated_${key}`, '1');
    localStorage.removeItem(key);
    logger.info(`[MIGRATE] Migration complete for key "${key}"`);
    return localStorageValue;
  }
  
  logger.info(`[MIGRATE] No localStorage value, reading from web.db`);
  return await webStorageGet(key);
}

export async function setLocale(locale: Locale): Promise<void> {
  logger.info(`[I18N] Setting locale to: "${locale}"`);
  currentLocale = locale;
  await webStorageSet('locale', locale);
  document.documentElement.lang = locale;
  logger.info(`[I18N] Locale "${locale}" set and saved`);
}

export function getLocale(): Locale {
  return currentLocale;
}

export async function initLocale(): Promise<Locale> {
  logger.info(`[I18N] ====== Initializing Locale ======`);
  const saved = await migrateFromLocalStorage('locale') as Locale | null;
  logger.info(`[I18N] Saved locale from storage: "${saved}"`);
  
  if (saved && locales[saved]) {
    logger.info(`[I18N] Applying saved locale: "${saved}"`);
    currentLocale = saved;
    document.documentElement.lang = saved;
    logger.info(`[I18N] ====== Locale initialized to "${saved}" ======`);
    return saved;
  }
  
  const browserLang = navigator.language;
  logger.info(`[I18N] No saved locale, using browser language: "${browserLang}"`);
  
  if (browserLang.startsWith('zh')) {
    currentLocale = 'zh-CN';
  } else if (browserLang.startsWith('ja')) {
    currentLocale = 'ja';
  } else {
    currentLocale = 'en';
  }
  
  document.documentElement.lang = currentLocale;
  logger.info(`[I18N] ====== Locale initialized to "${currentLocale}" (browser default) ======`);
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
