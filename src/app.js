const electron = require('electron');
const AutoLaunch = require('auto-launch');
const contribution = require('contribution');
const { CronJob, CronTime } = require('cron');

const icon = require('./icon');
const pjson = require('../package.json');
const store = require('./store');

const {
  app,
  BrowserWindow,
  Tray,
  Menu,
  shell,
  ipcMain,
  Notification,
} = electron;

app.on('ready', () => {
  const streakerAutoLauncher = new AutoLaunch({
    name: pjson.name,
  });

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

  function createTrayMenu(contributionCount, currentStreak, bestStreak) {
    const username = store.get('username') || 'Username not set';
    const githubProfileUrl = `https://github.com/${username}`;
    const menuTemplate = [
      { label: username, enabled: false },
      { type: 'separator' },
      { label: `Current Streak: ${currentStreak}`, enabled: false },
      { label: `Best Streak: ${bestStreak}`, enabled: false },
      { label: `Contributions: ${contributionCount}`, enabled: false },
      { type: 'separator' },
      {
        label: 'Reload',
        accelerator: 'CmdOrCtrl+R',
        click: requestContributionData,
      },
      {
        label: 'Open GitHub Profile...',
        accelerator: 'CmdOrCtrl+O',
        click: () => shell.openExternal(githubProfileUrl),
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
    ];
    return Menu.buildFromTemplate(menuTemplate);
  }

  function requestContributionData() {
    const username = store.get('username');
    if (!username) {
      tray.setImage(icon.fail);
      tray.setContextMenu(createTrayMenu(0, 0, 0));
      createPreferencesWindow();
      return;
    }

    tray.setImage(icon.load);
    tray.setContextMenu(
      createTrayMenu('Loading...', 'Loading...', 'Loading...'),
    );

    setTimeout(requestContributionData, 1000 * 60 * store.get('syncInterval')); // `syncInterval` minutes

    contribution(username)
      .then(data => {
        tray.setContextMenu(
          createTrayMenu(
            data.contributions,
            data.currentStreak,
            data.bestStreak,
          ),
        );
        tray.setImage(data.currentStreak > 0 ? icon.done : icon.todo);
      })
      .catch(error => {
        tray.setContextMenu(createTrayMenu('Error', 'Error', 'Error'));
        tray.setImage(icon.fail);
      });
  }

  async function setUsername(event, username) {
    if (username !== '' && username !== store.get('username')) {
      try {
        const userExist = await contribution(username);
        store.set('username', username);
        requestContributionData();
        event.sender.send('usernameSet', true);
      } catch (error) {
        event.sender.send('usernameSet', false);
      }
    }
  }

  function setSyncInterval(event, interval) {
    store.set('syncInterval', interval);
    event.sender.send('syncIntervalSet');
  }

  function activateLaunchAtLogin(event, isEnabled) {
    store.set('autoLaunch', isEnabled);
    if (isEnabled) {
      streakerAutoLauncher.enable();
    } else {
      streakerAutoLauncher.disable();
    }
    event.sender.send('activateLaunchAtLoginSet');
  }

  function activateNotifications(event, isEnabled) {
    store.set('notification.isEnabled', isEnabled);
    if (isEnabled) {
      job.setTime(
        new CronTime(
          `0 ${store.get('notification.minutes')} ${store.get(
            'notification.hours',
          )} * * *`,
        ),
      );
      job.start();
    } else {
      job.stop();
    }
    event.sender.send('activateNotificationsSet');
  }

  function setNotificationTime(event, data) {
    const { hours, minutes } = data;
    store.set('notification.hours', hours);
    store.set('notification.minutes', minutes);
    job.setTime(new CronTime(`0 ${minutes} ${hours} * * *`));
    job.start();
    event.sender.send('NotificationTimeSet');
  }

  const job = new CronJob({
    cronTime: '0 0 20 * * *',
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
    job.setTime(
      new CronTime(
        `0 ${store.get('notification.minutes')} ${store.get(
          'notification.hours',
        )} * * *`,
      ),
    );
    job.start();
  }

  process.on('uncaughtException', () => {
    tray.setContextMenu(createTrayMenu('Error', 'Error', 'Error'));
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
});
