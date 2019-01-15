const electron = require('electron');
const AutoLaunch = require('auto-launch');
const contribution = require('contribution');
const { CronJob, CronTime } = require('cron');

const icon = require('./icon');
const store = require('./store');
const pjson = require('../package.json');

const {
  app,
  BrowserWindow,
  ipcMain,
  Menu,
  Notification,
  shell,
  Tray,
} = electron;

app.on('ready', () => {
  const autoLauncher = new AutoLaunch({ name: pjson.name });
  const tray = new Tray(icon.done);
  let preferencesWindow = null;

  function createPreferencesWindow() {
    preferencesWindow = new BrowserWindow({
      title: `${pjson.name} - Preferences`,
      width: 300,
      height: 320,
      resizable: false,
      maximizable: false,
      minimizable: false,
      fullscreenable: false,
      show: false,
    });
    preferencesWindow.loadURL(
      `file://${__dirname}/preferences/preferences.html`,
    );

    preferencesWindow.on('ready-to-show', () => {
      preferencesWindow.show();
      if (process.platform === 'darwin') {
        app.dock.show();
      }
    });

    preferencesWindow.on('closed', () => {
      preferencesWindow = null;
      if (process.platform === 'darwin') {
        app.dock.hide();
      }
    });
  }

  function onPreferencesClick() {
    if (preferencesWindow === null) {
      return createPreferencesWindow();
    }
    preferencesWindow.focus();
  }

  function createTrayMenu(data = { streak: {}, contributions: {} }) {
    return Menu.buildFromTemplate([
      {
        label: store.get('username') || 'Set username...',
        enabled: !store.get('username'),
        click: onPreferencesClick,
      },
      { type: 'separator' },
      { label: 'Streak:', enabled: false },
      { label: `    Best: ${data.streak.best || 0}`, enabled: false },
      { label: `    Current: ${data.streak.current || 0}`, enabled: false },
      { label: 'Contributions:', enabled: false },
      { label: `    Best: ${data.contributions.best || 0}`, enabled: false },
      {
        label: `    Current: ${data.contributions.current || 0}`,
        enabled: false,
      },
      {
        label: `    Total: ${data.contributions.total || 0}`,
        enabled: false,
      },
      { type: 'separator' },
      {
        label: 'Reload',
        enabled: !!store.get('username'),
        accelerator: 'CmdOrCtrl+R',
        click: requestContributionData,
      },
      {
        label: 'Open GitHub Profile...',
        enabled: !!store.get('username'),
        accelerator: 'CmdOrCtrl+O',
        click: () =>
          shell.openExternal(`https://github.com/${store.get('username')}`),
      },
      { type: 'separator' },
      {
        label: 'Preferences...',
        accelerator: 'CmdOrCtrl+,',
        click: onPreferencesClick,
      },
      {
        label: `About ${pjson.name}...`,
        click: () => shell.openExternal(pjson.homepage),
      },
      {
        label: 'Feedback && Support...',
        click: () => shell.openExternal(pjson.bugs.url),
      },
      { type: 'separator' },
      {
        label: `Quit ${pjson.name}`,
        accelerator: 'CmdOrCtrl+Q',
        click: () => app.quit(),
      },
    ]);
  }

  async function requestContributionData() {
    const username = store.get('username');
    if (!username) {
      tray.setImage(icon.fail);
      tray.setContextMenu(createTrayMenu());
      return;
    }

    tray.setImage(icon.load);
    tray.setContextMenu(createTrayMenu());

    setTimeout(requestContributionData, 1000 * 60 * store.get('syncInterval'));

    try {
      const data = await contribution(username);
      tray.setContextMenu(createTrayMenu(data));
      tray.setImage(data.currentStreak > 0 ? icon.done : icon.todo);
      return data;
    } catch (error) {
      tray.setContextMenu(createTrayMenu());
      tray.setImage(icon.fail);
      throw new Error(error);
    }
  }

  async function setUsername(event, username) {
    try {
      store.set('username', username);
      await requestContributionData();
      store.set('userExists', !!username);
      event.sender.send('usernameSet', !!username);
    } catch (error) {
      store.set('userExists', false);
      event.sender.send('usernameSet', false);
    }
  }

  function setSyncInterval(event, interval) {
    store.set('syncInterval', interval);
    event.sender.send('syncIntervalSet');
  }

  function activateLaunchAtLogin(event, isEnabled) {
    store.set('autoLaunch', isEnabled);
    isEnabled ? autoLauncher.enable() : autoLauncher.disable();
    event.sender.send('activateLaunchAtLoginSet');
  }

  function activateNotifications(event, isEnabled) {
    store.set('notification.isEnabled', isEnabled);
    if (isEnabled) {
      const time = store.get('notification.time');
      const timeArray = time.split(':');
      const cronTime = `0 ${timeArray[1]} ${timeArray[0]} * * *`;
      job.setTime(new CronTime(cronTime));
      job.start();
    } else {
      job.stop();
    }
    event.sender.send('activateNotificationsSet');
  }

  function setNotificationTime(event, time) {
    store.set('notification.time', time);
    const timeArray = time.split(':');
    const cronTime = `0 ${timeArray[1]} ${timeArray[0]} * * *`;
    job.setTime(new CronTime(cronTime));
    job.start();
    event.sender.send('NotificationTimeSet');
  }

  const job = new CronJob({
    cronTime: '0 0 20 00 * *',
    async onTick() {
      const data = await contribution(store.get('username'));
      if (data.currentStreak === 0 && Notification.isSupported()) {
        new Notification({
          title: pjson.name,
          body: "You haven't contributed today",
        }).show();
      }
    },
  });

  if (store.get('notification.isEnabled')) {
    const time = store.get('notification.time');
    const timeArray = time.split(':');
    const cronTime = `0 ${timeArray[1]} ${timeArray[0]} * * *`;
    job.setTime(new CronTime(cronTime));
    job.start();
  }

  process.on('uncaughtException', () => {
    tray.setContextMenu(createTrayMenu());
    tray.setImage(icon.fail);
  });

  if (process.platform === 'darwin') {
    app.dock.hide();
  }

  app.on('window-all-closed', () => {});
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
