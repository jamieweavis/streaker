import url from 'url';
import path from 'path';
import AutoLaunch from 'auto-launch';
import { fetchStats } from 'contribution';
import { CronJob, CronTime } from 'cron';
import {
  app,
  BrowserWindow,
  ipcMain,
  Notification,
  shell,
  Tray,
  IpcMainEvent,
} from 'electron';

import { createMenu } from './menu';
import icons from './icons';
import store from '../common/store';
import { PreferencesSavedValues } from '../common/types';
import pjson from '../../package.json';

const bootstrap = (): void => {
  const autoLaunch = new AutoLaunch({ name: 'Streaker' });
  const tray = new Tray(icons[store.get('iconSet')].contributed);
  let preferencesWindow: Electron.BrowserWindow | null = null;

  const createPreferencesWindow = (): void => {
    preferencesWindow = new BrowserWindow({
      title: 'Streaker',
      width: 290,
      height: 500,
      resizable: false,
      maximizable: false,
      minimizable: false,
      fullscreenable: false,
      show: false,
      webPreferences: {
        nodeIntegration: true,
      },
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

  const reminderNotificationJob = new CronJob({
    cronTime: '0 0 20 * * *',
    onTick: async (): Promise<void> => {
      if (!Notification.isSupported()) {
        return;
      }
      const stats = await fetchStats(store.get('username'));
      if (stats.contributions.current === 0) {
        new Notification({
          title: "🔥 You haven't contributed today",
          body:
            "Don't forget to contribute today to continue your streak on GitHub!",
        }).show();
      }
    },
  });

  const onPreferencesClick = (): void => {
    if (preferencesWindow === null) {
      return createPreferencesWindow();
    }
    preferencesWindow.focus();
  };

  const onAboutClick = (): Promise<void> => shell.openExternal(pjson.homepage);

  const onProfileClick = (): Promise<void> =>
    shell.openExternal(`https://github.com/${store.get('username')}`);

  const onFeedbackClick = (): Promise<void> =>
    shell.openExternal(pjson.bugs.url);

  const onQuitClick = (): void => app.quit();

  const setTrayMenu = (username?, stats?): void => {
    const menu = createMenu({
      onPreferencesClick,
      onAboutClick,
      onQuitClick,
      onProfileClick,
      onFeedbackClick,
      requestContributionData, // eslint-disable-line
      stats,
      username,
    });
    tray.setContextMenu(menu);
  };

  const requestContributionData = async (): Promise<void> => {
    setTimeout(requestContributionData, 1000 * 60 * store.get('syncInterval'));
    tray.setImage(icons[store.get('iconSet')].loading);

    const username = store.get('username');
    if (!username) {
      tray.setImage(icons[store.get('iconSet')].error);
      setTrayMenu();
      return;
    }

    let stats;
    try {
      stats = await fetchStats(username);
      tray.setImage(
        stats?.streak?.current > 0
          ? icons[store.get('iconSet')].contributed
          : icons[store.get('iconSet')].pending,
      );
    } catch (err) {
      tray.setImage(icons[store.get('iconSet')].error);
    }

    setTrayMenu(username, stats);
  };

  const onPreferencesSaved = (
    event: IpcMainEvent,
    args: PreferencesSavedValues,
  ): void => {
    if (args.launchAtLogin !== store.get('launchAtLogin')) {
      args.launchAtLogin ? autoLaunch.enable() : autoLaunch.disable();
    }
    if (
      args.notificationEnabled !== store.get('notificationEnabled') ||
      args.notificationTime !== store.get('notificationTime')
    ) {
      if (args.notificationEnabled) {
        const [hours, minutes] = args.notificationTime.split(':');
        const cronTime = `0 ${minutes} ${hours} * * *`;
        reminderNotificationJob.setTime(new CronTime(cronTime));
        reminderNotificationJob.start();
      } else {
        reminderNotificationJob.stop();
      }
    }
    store.set(args);
    requestContributionData();
  };

  if (store.get('notificationEnabled')) {
    const notificationTime = store.get('notificationTime');
    const [hours, minutes] = notificationTime.split(':');
    const cronTime = `0 ${minutes} ${hours} * * *`;
    reminderNotificationJob.setTime(new CronTime(cronTime));
    reminderNotificationJob.start();
  }

  tray.on('right-click', requestContributionData);

  ipcMain.on('preferences-saved', onPreferencesSaved);

  app.on('window-all-closed', () => {
    /* Keep open on macos */
  });

  if (process.platform === 'darwin') {
    app.dock.hide();
  }

  if (!store.get('username')) createPreferencesWindow();

  requestContributionData();

  process.on('uncaughtException', () => {
    const menu = createMenu({
      onPreferencesClick,
      onAboutClick,
      onQuitClick,
      onProfileClick,
      onFeedbackClick,
      requestContributionData,
      username: store.get('username'),
    });
    tray.setContextMenu(menu);
    tray.setImage(icons[store.get('iconSet')].error);
  });
};

app.whenReady().then(bootstrap);
