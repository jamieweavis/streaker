import AutoLaunch from 'auto-launch';
import { type GitHubStats, fetchStats } from 'contribution';
import { CronJob, CronTime } from 'cron';
import type { NativeImage } from 'electron';
import { BrowserWindow, Notification, Tray, app, ipcMain } from 'electron';
import { UpdateSourceType, updateElectronApp } from 'update-electron-app';

import { icons } from '../icons';
import { logger } from './logger';
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
  const { username, iconTheme, reminderTime, reminderEnabled } = store.store;

  let preferencesWindow: BrowserWindow | null = null;
  let lastFetchedStats: GitHubStats | undefined;

  const tray = new Tray(icons[iconTheme].pending);

  const createPreferencesWindow = () => {
    preferencesWindow = new BrowserWindow({
      width: 310,
      height: 485,
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
      if (preferencesWindow) {
        preferencesWindow.show();
        if (isDev) preferencesWindow.webContents.openDevTools();
      }
      if (process.platform === 'darwin') app.dock.show();
    });

    preferencesWindow.on('closed', () => {
      preferencesWindow = null;
      if (process.platform === 'darwin') app.dock.hide();
    });
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
    const username = store.get('username');
    const iconTheme = store.get('iconTheme');

    if (!username) {
      return createTray({ icon: icons[iconTheme].error });
    }

    let stats: GitHubStats | undefined;
    let icon = icons[iconTheme].pending;

    try {
      stats = await fetchStats(username);
      logger.info('Fetched contribution stats', stats);

      lastFetchedStats = stats;

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
    logger.info('Saving preferences:', preferences);

    // Update auto launch
    try {
      if (preferences.launchAtLogin !== store.get('launchAtLogin')) {
        if (preferences.launchAtLogin) {
          autoLaunch.enable();
          logger.info('Auto launch enabled');
        } else {
          autoLaunch.disable();
          logger.info('Auto launch disabled');
        }
      }
    } catch (error) {
      logger.error('Error enabling auto launch', error);
    }

    // Update reminder notification
    try {
      if (
        preferences.reminderEnabled !== store.get('reminderEnabled') ||
        preferences.reminderTime !== store.get('reminderTime')
      ) {
        if (preferences.reminderEnabled) {
          const [hours, minutes] = preferences.reminderTime.split(':');
          const cronTime = new CronTime(`${minutes} ${hours} * * *`);
          reminderNotification.setTime(cronTime);
          reminderNotification.start();
          logger.info(
            'Reminder notification enabled:',
            preferences.reminderTime,
          );
        } else {
          reminderNotification.stop();
          logger.info('Reminder notification disabled');
        }
      }
    } catch (error) {
      logger.error('Error enabling reminder notification', error);
    }

    // Save preferences
    store.set(preferences);
    logger.info('Preferences saved');
  };

  const showReminderNotification = () => {
    logger.info('Triggering reminder notification');

    try {
      if (!Notification.isSupported()) {
        logger.error('Notification not supported');
        return;
      }
      if (!lastFetchedStats.streak.isAtRisk) {
        logger.info('Streak not at risk, skipping reminder notification');
        return;
      }
      new Notification({
        title: `🔥 Don't lose your streak!`,
        body: `Contribute today to continue your ${lastFetchedStats.streak.previous} day streak on GitHub`,
      }).show();
    } catch (error) {
      logger.error('Error showing reminder notification', error);
    }
  };

  app.on('window-all-closed', () => {});
  if (process.platform === 'darwin') app.dock.hide();

  app.on('activate', () => {
    if (!preferencesWindow) createPreferencesWindow();
  });

  // Fetch data on app start and every minute
  fetchContributionStats();
  new CronJob('* * * * *', fetchContributionStats, null, true);

  // Handle saving preferences
  ipcMain.on('savePreferences', (event, preferences: Preferences) => {
    savePreferences(preferences);
    fetchContributionStats();
  });

  // Set up reminder notification
  const [hours, minutes] = reminderTime.split(':');
  const reminderNotification = new CronJob(
    `${minutes} ${hours} * * *`,
    showReminderNotification,
  );
  if (reminderEnabled) reminderNotification.start();

  // Set up auto launch
  const autoLaunch = new AutoLaunch({
    name: 'Streaker',
    isHidden: true,
  });

  // Set up auto updater
  updateElectronApp({
    updateSource: {
      type: UpdateSourceType.ElectronPublicUpdateService,
      repo: 'jamieweavis/streaker',
    },
    updateInterval: '1 hour',
    logger: logger,
  });

  // Open preferences window automatically if username is not (most likely fresh install)
  if (!username || isDev) createPreferencesWindow();
};

app
  .whenReady()
  .then(bootstrap)
  .catch((error) => {
    logger.error('An error occurred', error);
  });
