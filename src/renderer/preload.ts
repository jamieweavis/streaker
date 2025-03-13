import { contextBridge, ipcRenderer } from 'electron';

import type { Preferences } from '../main/main';

contextBridge.exposeInMainWorld('streaker', {
  initialPreferences: () => {
    const preferences: Record<string, unknown> = {};
    for (const arg of process.argv) {
      const [key, value] = arg.split('=');
      if (key && value) {
        preferences[key.substring(2)] = value;
      }
    }
    return preferences;
  },
  savePreferences: (preferences: Preferences) =>
    ipcRenderer.send('savePreferences', preferences),
});
