import url from 'url';
import path from 'path';
import AutoLaunch from 'auto-launch';
import { fetchStats, GitHubStats } from 'contribution';
import { CronJob, CronTime } from 'cron';
import {
  app,
  BrowserWindow,
  ipcMain,
  Notification,
  Tray,
  IpcMainEvent,
} from 'electron';

import store from '@common/store';
import { PreferencesSavedValues } from '@common/types';
import { createMenu } from '@main/menu';
import iconSets from '@main/icons';

const bootstrap = (): void => {
  const autoLaunch = new AutoLaunch({ name: 'Streaker', isHidden: true });
  const tray = new Tray(iconSets[store.get('iconSet')].contributed);
  const reminderNotificationJob = new CronJob({
    cronTime: '0 0 20 * * *',
    onTick: handleReminderNotification,
  });

  let preferencesWindow: Electron.BrowserWindow;

  const createPreferencesWindow = (): void => {
    preferencesWindow = new BrowserWindow({
      title: 'Streaker',
      width: 290,
      height: 615,
      resizable: false,
      maximizable: false,
      minimizable: false,
      fullscreenable: false,
      show: false,
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
      requestContributionData,
      preferencesWindow,
      createPreferencesWindow,
      app,
    });
    tray.setContextMenu(menu);
    if (icon) tray.setImage(icon);
  };

  const requestContributionData = async (): Promise<void> => {
    const iconSet = store.get('iconSet');
    const username = store.get('username');

    if (!username) {
      const icon = iconSets[iconSet].error;
      setTrayMenu({ icon });
      return;
    }

    tray.setImage(iconSets[iconSet].loading);

    let stats: GitHubStats;
    let icon = iconSets[iconSet].pending;
    try {
      stats = await fetchStats(username);
      if (stats?.streak?.current > 0) icon = iconSets[iconSet].contributed;
    } catch (err) {
      icon = iconSets[iconSet].error;
    }

    setTrayMenu({ username, stats, icon });
    setTimeout(requestContributionData, 1000 * 60 * store.get('pollInterval'));
  };

  const onPreferencesSaved = (
    event: IpcMainEvent,
    values: PreferencesSavedValues,
  ): void => {
    if (values.launchAtLogin !== store.get('launchAtLogin')) {
      values.launchAtLogin ? autoLaunch.enable() : autoLaunch.disable();
    }
    if (
      values.reminderEnabled !== store.get('reminderEnabled') ||
      values.reminderTime !== store.get('reminderTime')
    ) {
      if (values.reminderEnabled) {
        const [hours, minutes] = values.reminderTime.split(':');
        const cronTime = `0 ${minutes} ${hours} * * *`;
        reminderNotificationJob.setTime(new CronTime(cronTime));
        reminderNotificationJob.start();
      } else {
        reminderNotificationJob.stop();
      }
    }
    store.set(values);
    requestContributionData();
  };

  if (store.get('reminderEnabled')) {
    const reminderTime = store.get('reminderTime');
    const [hours, minutes] = reminderTime.split(':');
    const cronTime = `0 ${minutes} ${hours} * * *`;
    reminderNotificationJob.setTime(new CronTime(cronTime));
    reminderNotificationJob.start();
  }

  tray.on('right-click', requestContributionData);

  ipcMain.on('preferences-saved', onPreferencesSaved);

  // Keep app open if all windows are closed
  app.on('window-all-closed', () => {}); // eslint-disable-line

  // Hide dock icon on macOS
  if (process.platform === 'darwin') app.dock.hide();

  // Open preferences window if no username is set (probably initial launch)
  if (!store.get('username')) createPreferencesWindow();

  // Initial request
  requestContributionData();

  process.on('uncaughtException', () => {
    const username = store.get('username');
    const iconSet = store.get('iconSet');
    const icon = iconSets[iconSet].error;

    setTrayMenu({ username, icon });
  });
};

const handleReminderNotification = async (): Promise<void> => {
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
