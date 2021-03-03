import url from 'url';
import path from 'path';
import AutoLaunch from 'auto-launch';
import { CronJob, CronTime } from 'cron';
import { fetchStats, GitHubStats } from 'contribution';
import { app, BrowserWindow, ipcMain, Notification, Tray } from 'electron';

import store from '@common/store';
import iconSets from '@main/icons';
import { createMenu } from '@main/menu';
import { PreferencesSavedValues } from '@common/types';

const bootstrap = (): void => {
  let preferencesWindow: Electron.BrowserWindow;

  const createPreferencesWindow = (): void => {
    preferencesWindow = new BrowserWindow({
      width: 300,
      height: 625,
      show: false,
      resizable: false,
      maximizable: false,
      minimizable: false,
      fullscreenable: false,
      webPreferences: { nodeIntegration: true },
    });

    if (process.env.NODE_ENV !== 'production') {
      preferencesWindow.loadURL(`http://localhost:3000`);
    } else {
      const pathname = path.join(__dirname, 'index.html');
      preferencesWindow.loadURL(
        url.format({ pathname, protocol: 'file:', slashes: true }),
      );
    }

    preferencesWindow.on('ready-to-show', () => {
      if (preferencesWindow) preferencesWindow.show();
      if (process.platform === 'darwin') app.dock.show();
    });

    preferencesWindow.on('closed', () => {
      preferencesWindow = null;
      if (process.platform === 'darwin') app.dock.hide();
    });
  };

  const setTrayMenu = ({
    username,
    stats,
    icon,
  }: {
    username?: string;
    stats?: GitHubStats;
    icon?: string;
  }): void => {
    const menu = createMenu({
      username,
      stats,
      createPreferencesWindow,
      requestContributionData,
      isPreferencesWindowOpen: () => !!preferencesWindow,
      focusPreferencesWindow: () => preferencesWindow.focus(),
    });
    tray.setContextMenu(menu);
    if (icon) tray.setImage(icon);
  };

  const requestContributionData = async (): Promise<void> => {
    const iconSet = store.get('iconSet');
    const username = store.get('username');

    if (!username) return setTrayMenu({ icon: iconSets[iconSet].error });
    tray.setImage(iconSets[iconSet].loading);

    let stats: GitHubStats;
    let icon = iconSets[iconSet].pending;
    try {
      stats = await fetchStats(username);
      if (stats?.streak?.current > 0) icon = iconSets[iconSet].contributed;
      if (stats?.streak?.current === stats?.streak?.best)
        icon = iconSets[iconSet].streaking;
    } catch (err) {
      icon = iconSets[iconSet].error;
    }

    setTrayMenu({ username, stats, icon });
  };

  const onPreferencesSaved = (event, values: PreferencesSavedValues): void => {
    if (values.launchAtLogin !== store.get('launchAtLogin')) {
      values.launchAtLogin ? autoLaunch.enable() : autoLaunch.disable();
    }
    if (
      values.reminderEnabled !== store.get('reminderEnabled') ||
      values.reminderTime !== store.get('reminderTime')
    ) {
      if (values.reminderEnabled) {
        const [hours, minutes] = values.reminderTime.split(':');
        const cronTime = new CronTime(`0 ${minutes} ${hours} * * *`);
        reminderNotification.setTime(cronTime);
        reminderNotification.start();
      } else {
        reminderNotification.stop();
      }
    }
    store.set(values);
    requestContributionData();
  };

  // Setup tray
  const tray = new Tray(iconSets[store.get('iconSet')].loading);
  tray.on('right-click', requestContributionData);

  // Auto launch
  const autoLaunch = new AutoLaunch({ name: 'Streaker', isHidden: true });

  // Reminder notification
  const reminderEnabled = store.get('reminderEnabled');
  const reminderTime = store.get('reminderTime');
  const [hours, minutes] = reminderTime.split(':');
  const reminderNotification = new CronJob({
    cronTime: `0 ${minutes} ${hours} * * *`,
    onTick: triggerReminderNotification,
  });
  if (reminderEnabled) reminderNotification.start();

  // Initial request & cron sync every 15 minutes
  requestContributionData();
  new CronJob({
    cronTime: '*/15 * * * *',
    onTick: requestContributionData,
  }).start();

  // Handle uncaught errors gracefully
  process.on('uncaughtException', () => {
    const username = store.get('username');
    const iconSet = store.get('iconSet');
    const icon = iconSets[iconSet].error;
    setTrayMenu({ username, icon });
  });

  // Platform specific app handling
  app.on('window-all-closed', () => {}); // eslint-disable-line
  if (process.platform === 'darwin') app.dock.hide();

  // Save preferences to store on renderer preferences save
  ipcMain.on('preferences-saved', onPreferencesSaved);

  // Open preferences window if no username is set (probably initial launch)
  if (!store.get('username')) createPreferencesWindow();
};

const triggerReminderNotification = async (): Promise<void> => {
  if (!Notification.isSupported()) return;
  const username = store.get('username');
  const stats = await fetchStats(username);
  if (stats?.contributions?.current === 0) {
    new Notification({
      title: "🔥 You haven't contributed today",
      body:
        "Don't forget to contribute today to continue your streak on GitHub!",
    }).show();
  }
};

app.whenReady().then(bootstrap);
