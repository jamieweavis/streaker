import AutoLaunch from 'auto-launch';
import { type GitHubStats, fetchStats } from 'contribution';
import { CronJob, CronTime } from 'cron';
import type { NativeImage } from 'electron';
import {
  BrowserWindow,
  Notification,
  Tray,
  app,
  dialog,
  ipcMain,
  shell,
} from 'electron';

import { icons } from '../icons';
import MenuBuilder from './menu';
import { store } from './store';
import { createTrayMenu } from './tray-menu';

declare const MAIN_WINDOW_WEBPACK_ENTRY: string;
declare const MAIN_WINDOW_PRELOAD_WEBPACK_ENTRY: string;

export interface Preferences {
  username: string;
  launchAtLogin: boolean;
  reminderEnabled: boolean;
  reminderTime: string;
  iconTheme: string;
}

const isDev = process.env.NODE_ENV === 'development';

const bootstrap = (): void => {
  let preferencesWindow: BrowserWindow | null = null;
  const iconTheme = store.get('iconTheme') as keyof typeof icons;
  const tray = new Tray(icons[iconTheme].pending);
  let lastStats: GitHubStats | undefined;

  const createPreferencesWindow = () => {
    preferencesWindow = new BrowserWindow({
      width: 300,
      height: 480,
      show: false,
      resizable: false,
      maximizable: false,
      minimizable: false,
      fullscreenable: false,
      frame: false,
      webPreferences: {
        preload: MAIN_WINDOW_PRELOAD_WEBPACK_ENTRY,
        additionalArguments: Object.entries(store.store).map(
          ([key, value]) => `--${key}=${value}`,
        ),
      },
    });

    preferencesWindow.loadURL(MAIN_WINDOW_WEBPACK_ENTRY);

    const menuBuilder = new MenuBuilder(preferencesWindow);
    menuBuilder.buildMenu();

    preferencesWindow.on('ready-to-show', () => {
      if (preferencesWindow) preferencesWindow.show();
      if (process.platform === 'darwin') app.dock.show();
    });

    preferencesWindow.on('closed', () => {
      preferencesWindow = null;
      if (process.platform === 'darwin') app.dock.hide();
    });

    // Open urls in the user's browser
    preferencesWindow.webContents.setWindowOpenHandler((edata) => {
      shell.openExternal(edata.url);
      return { action: 'deny' };
    });

    if (isDev) preferencesWindow.webContents.openDevTools();
  };

  const createTray = ({
    username,
    stats,
    icon,
  }: {
    username?: string;
    stats?: GitHubStats;
    icon?: NativeImage;
  }) => {
    const trayMenu = createTrayMenu({
      username,
      stats,
      createPreferencesWindow: createPreferencesWindow,
      requestContributionData: fetchContributionStats,
      isPreferencesWindowOpen: () => !!preferencesWindow,
      focusPreferencesWindow: () => {
        if (!preferencesWindow) return;
        if (preferencesWindow.isMinimized()) preferencesWindow.restore();
        preferencesWindow.focus();
      },
    });
    tray.setContextMenu(trayMenu);
    if (icon) tray.setImage(icon);
  };

  const fetchContributionStats = async () => {
    let stats: GitHubStats | undefined;

    const username = store.get('username') as string;
    const iconTheme = store.get('iconTheme') as keyof typeof icons;

    if (!username) {
      return createTray({ icon: icons[iconTheme].error });
    }

    let icon = icons[iconTheme].pending;
    try {
      stats = await fetchStats(username);
      lastStats = stats;
      console.info('Fetched contribution stats', stats);
      if (stats.streak.current > 0) {
        icon = icons[iconTheme].contributed;
      }
      if (stats.streak.current === stats.streak.best) {
        icon = icons[iconTheme].streaking;
      }
    } catch (err) {
      icon = icons[iconTheme].error;
    }

    createTray({ username, stats, icon });
  };

  const savePreferences = (preferences: Preferences) => {
    console.info('Saving preferences', preferences);

    // Update auto launch
    if (preferences.launchAtLogin !== store.get('launchAtLogin')) {
      if (preferences.launchAtLogin) {
        autoLaunch.enable();
        console.info('Auto launch enabled');
      } else {
        autoLaunch.disable();
        console.info('Auto launch disabled');
      }
    }

    // Update reminder notification
    if (
      preferences.reminderEnabled !== store.get('reminderEnabled') ||
      preferences.reminderTime !== store.get('reminderTime')
    ) {
      if (preferences.reminderEnabled) {
        const [hours, minutes] = preferences.reminderTime.split(':');
        const cronTime = new CronTime(`${minutes} ${hours} * * *`);
        reminderNotification.setTime(cronTime);
        reminderNotification.start();
        console.info('Reminder notification enabled');
      } else {
        reminderNotification.stop();
        console.info('Reminder notification disabled');
      }
    }

    // Save preferences
    store.set(preferences);
  };

  const showReminderNotification = () => {
    console.info('Triggering reminder notification');
    if (!Notification.isSupported()) {
      console.error('Notification not supported');
      return;
    }
    if (!lastStats.streak.isAtRisk) {
      console.info('Notification cancelled, streak not at risk');
      return;
    }
    new Notification({
      title: `🔥 Don't lose your streak!`,
      body: `Contribute today to continue your ${lastStats.streak.previous} day streak on GitHub`,
    }).show();
  };

  app.on('window-all-closed', () => {});
  if (process.platform === 'darwin') app.dock.hide();

  // Fetch data on app start and every minute
  fetchContributionStats();
  new CronJob('* * * * *', fetchContributionStats, null, true);

  // Handle saving preferences
  ipcMain.on('savePreferences', (event, preferences: Preferences) => {
    const oldUsername = store.get('username');
    savePreferences(preferences);
    fetchContributionStats();
  });

  // Set up reminder notification
  const reminderTime = store.get('reminderTime');
  const [hours, minutes] = reminderTime.split(':');
  const reminderNotification = new CronJob(
    `${minutes} ${hours} * * *`,
    showReminderNotification,
  );
  const reminderEnabled = store.get('reminderEnabled');
  if (reminderEnabled) reminderNotification.start();

  // Set up auto launch
  const autoLaunch = new AutoLaunch({
    name: 'Streaker',
    isHidden: true,
  });

  // Open preferences window automatically if username is not (most likely fresh install)
  const username = store.get('username');
  if (!username || isDev) {
    createPreferencesWindow();
  }
};

app
  .whenReady()
  .then(bootstrap)
  .catch((error) => {
    console.error('An error occurred', error);
    dialog.showErrorBox('An error occurred', error.message);
  });
