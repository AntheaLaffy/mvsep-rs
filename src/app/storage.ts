import { invoke } from '@tauri-apps/api/core';
import { logger } from './utils/logger';

async function webStorageGet(key: string): Promise<string | null> {
  try {
    logger.info(`[STORAGE] Getting key: "${key}"`);
    const result = await invoke('web_storage_get', { key }) as string | null;
    logger.info(`[STORAGE] Got key "${key}": ${result !== null ? `"${result}"` : 'null'}`);
    return result;
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

export async function readJsonStorage<T>(key: string, fallback: T): Promise<T> {
  try {
    logger.info(`[STORAGE] Reading JSON storage for key: "${key}"`);
    const raw = await migrateFromLocalStorage(key);
    if (!raw) {
      logger.info(`[STORAGE] No value for key "${key}", returning fallback`);
      return fallback;
    }
    const parsed = JSON.parse(raw) as T;
    logger.info(`[STORAGE] Successfully read JSON for key "${key}"`);
    return parsed;
  } catch (e) {
    logger.error(`[STORAGE] Error reading JSON for key "${key}": ${String(e)}, returning fallback`);
    return fallback;
  }
}

export async function writeJsonStorage<T>(key: string, value: T): Promise<boolean> {
  try {
    logger.info(`[STORAGE] Writing JSON storage for key: "${key}"`);
    const json = JSON.stringify(value);
    await invoke('web_storage_set', { key, value: json });
    logger.info(`[STORAGE] Successfully wrote JSON for key "${key}"`);
    return true;
  } catch (e) {
    logger.error(`[STORAGE] Error writing JSON for key "${key}": ${String(e)}`);
    return false;
  }
}

export async function readTextStorage(key: string): Promise<string | null> {
  try {
    logger.info(`[STORAGE] Reading text storage for key: "${key}"`);
    const result = await migrateFromLocalStorage(key);
    logger.info(`[STORAGE] Read text for key "${key}": ${result !== null ? `"${result}"` : 'null'}`);
    return result;
  } catch (e) {
    logger.error(`[STORAGE] Error reading text for key "${key}": ${String(e)}`);
    return null;
  }
}

export async function writeTextStorage(key: string, value: string): Promise<boolean> {
  try {
    logger.info(`[STORAGE] Writing text storage for key: "${key}", value: "${value}"`);
    await invoke('web_storage_set', { key, value });
    logger.info(`[STORAGE] Successfully wrote text for key "${key}"`);
    return true;
  } catch (e) {
    logger.error(`[STORAGE] Error writing text for key "${key}": ${String(e)}`);
    return false;
  }
}
