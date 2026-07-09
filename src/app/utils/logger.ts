let sendLogFn: ((level: 'INFO' | 'WARN' | 'ERROR' | 'DEBUG', message: string) => void) | null = null;

export function setLogSender(fn: (level: 'INFO' | 'WARN' | 'ERROR' | 'DEBUG', message: string) => void) {
  sendLogFn = fn;
}

function sendLog(level: 'INFO' | 'WARN' | 'ERROR' | 'DEBUG', message: string) {
  if (sendLogFn) {
    void sendLogFn(level, message);
  }
  switch (level) {
    case 'INFO':
      console.log(`[LOG] ${message}`);
      break;
    case 'WARN':
      console.warn(`[LOG] ${message}`);
      break;
    case 'ERROR':
      console.error(`[LOG] ${message}`);
      break;
    case 'DEBUG':
      console.debug(`[LOG] ${message}`);
      break;
  }
}

export const logger = {
  info(message: string) {
    sendLog('INFO', message);
  },
  warn(message: string) {
    sendLog('WARN', message);
  },
  error(message: string) {
    sendLog('ERROR', message);
  },
  debug(message: string) {
    sendLog('DEBUG', message);
  },
};