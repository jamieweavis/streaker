import url from 'url';
import path from 'path';
import AutoLaunch from 'auto-launch';
import { fetchStats } from 'contribution';
import { CronJob, CronTime } from 'cron';
import {
  app,
  BrowserWindow,
  ipcMain,
  Menu,
  Notification,
  shell,
  Tray,
  IpcMainEvent,
} from 'electron';

import icons from './icons';
import store from './store';
import pjson from '../../package.json';

app.on('ready', () => {
  const autoLauncher = new AutoLaunch({ name: 'Streaker' });
  const tray = new Tray(icons.done);
  let preferencesWindow: Electron.BrowserWindow | null = null;

  const createPreferencesWindow = (): void => {
    preferencesWindow = new BrowserWindow({
      title: 'Streaker',
      width: 300,
      height: 320,
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
      preferencesWindow.loadURL(
        url.format({
          pathname: path.join(__dirname, 'index.html'),
          protocol: 'file:',
          slashes: true,
        }),
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

  const onPreferencesClick = (): void => {
    if (preferencesWindow === null) {
      return createPreferencesWindow();
    }
    preferencesWindow.focus();
  };

  const createTrayMenu = (stats?): Menu =>
    Menu.buildFromTemplate([
      {
        label: store.get('username') || 'Set username...',
        enabled: !store.get('username'),
        click: onPreferencesClick,
      },
      { type: 'separator' },
      { label: 'Streak:', enabled: false },
      { label: `    Best: ${stats?.streak?.best || 0}`, enabled: false },
      {
        label: `    Current: ${stats?.streak?.current || 0}`,
        enabled: false,
      },
      { label: 'Contributions:', enabled: false },
      {
        label: `    Best: ${stats?.contributions?.best || 0}`,
        enabled: false,
      },
      {
        label: `    Current: ${stats?.contributions?.current || 0}`,
        enabled: false,
      },
      {
        label: `    Total: ${stats?.contributions?.total || 0}`,
        enabled: false,
      },
      { type: 'separator' },
      {
        label: 'Reload',
        enabled: !!store.get('username'),
        accelerator: 'CmdOrCtrl+R',
        click: requestContributionData, //eslint-disable-line
      },
      {
        label: 'Open GitHub Profile...',
        enabled: !!store.get('username'),
        accelerator: 'CmdOrCtrl+O',
        click: (): Promise<void> =>
          shell.openExternal(`https://github.com/${store.get('username')}`),
      },
      { type: 'separator' },
      {
        label: 'Preferences...',
        accelerator: 'CmdOrCtrl+,',
        click: onPreferencesClick,
      },
      {
        label: 'About Streaker...',
        click: (): Promise<void> => shell.openExternal(pjson.homepage),
      },
      {
        label: 'Feedback && Support...',
        click: (): Promise<void> => shell.openExternal(pjson.bugs.url),
      },
      { type: 'separator' },
      {
        label: 'Quit Streaker',
        accelerator: 'CmdOrCtrl+Q',
        click: (): void => app.quit(),
      },
    ]);

  const requestContributionData = async (): Promise<object> => {
    const username = store.get('username');
    if (!username) {
      tray.setImage(icons.fail);
      tray.setContextMenu(createTrayMenu());
      return;
    }

    tray.setImage(icons.load);
    tray.setContextMenu(createTrayMenu());

    setTimeout(requestContributionData, 1000 * 60 * store.get('syncInterval'));

    try {
      const stats = await fetchStats(username);
      tray.setContextMenu(createTrayMenu(stats));
      tray.setImage(stats.streak.current > 0 ? icons.done : icons.todo);
      return stats;
    } catch (error) {
      tray.setContextMenu(createTrayMenu());
      tray.setImage(icons.fail);
      throw new Error(error);
    }
  };

  const setUsername = async (
    event: IpcMainEvent,
    username: string,
  ): Promise<void> => {
    try {
      store.set('username', username);
      await requestContributionData();
      store.set('userExists', !!username);
      event.sender.send('usernameSet', !!username);
    } catch (error) {
      store.set('userExists', false);
      event.sender.send('usernameSet', false);
    }
  };

  const setSyncInterval = (event: IpcMainEvent, interval: number): void => {
    store.set('syncInterval', interval);
    event.sender.send('syncIntervalSet');
  };

  const activateLaunchAtLogin = (
    event: IpcMainEvent,
    isEnabled: boolean,
  ): void => {
    store.set('autoLaunch', isEnabled);
    isEnabled ? autoLauncher.enable() : autoLauncher.disable();
    event.sender.send('activateLaunchAtLoginSet');
  };

  const activateNotifications = (
    event: IpcMainEvent,
    isEnabled: boolean,
  ): void => {
    store.set('notificationEnabled', isEnabled);
    if (isEnabled) {
      const time = store.get('notificationTime');
      const timeArray = time.split(':');
      const cronTime = `0 ${timeArray[1]} ${timeArray[0]} * * *`;
      job.setTime(new CronTime(cronTime)); // eslint-disable-line
      job.start(); // eslint-disable-line
    } else {
      job.stop(); // eslint-disable-line
    }
    event.sender.send('activateNotificationsSet');
  };

  const setNotificationTime = (event: IpcMainEvent, time: string): void => {
    store.set('notificationTime', time);
    const timeArray = time.split(':');
    const cronTime = `0 ${timeArray[1]} ${timeArray[0]} * * *`;
    job.setTime(new CronTime(cronTime)); // eslint-disable-line
    job.start(); // eslint-disable-line
    event.sender.send('NotificationTimeSet');
  };

  const job = new CronJob({
    cronTime: '0 0 20 * * *',
    async onTick(): Promise<void> {
      const data = await fetchStats(store.get('username'));
      if (data.streak.current === 0 && Notification.isSupported()) {
        new Notification({
          title: 'Streaker',
          body: "Reminder: You haven't contributed today!",
        }).show();
      }
    },
  });

  const notificationTime = store.get('notificationTime');
  const notificationEnabled = store.get('notificationEnabled');
  if (notificationEnabled) {
    const timeArray = notificationTime.split(':');
    const cronTime = `0 ${timeArray[1]} ${timeArray[0]} * * *`;
    job.setTime(new CronTime(cronTime));
    job.start();
  }

  process.on('uncaughtException', () => {
    tray.setContextMenu(createTrayMenu());
    tray.setImage(icons.fail);
  });

  if (process.platform === 'darwin') {
    app.dock.hide();
  }

  app.on('window-all-closed', () => {}); // eslint-disable-line
  tray.on('right-click', requestContributionData);
  ipcMain.on('setUsername', setUsername);
  ipcMain.on('setSyncInterval', setSyncInterval);
  ipcMain.on('activateLaunchAtLogin', activateLaunchAtLogin);
  ipcMain.on('activateNotifications', activateNotifications);
  ipcMain.on('setNotificationTime', setNotificationTime);

  requestContributionData();

  if (!store.get('username')) {
    createPreferencesWindow();
  }
});
