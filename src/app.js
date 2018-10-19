const electron = require('electron');
const AutoLaunch = require('auto-launch');
const contribution = require('contribution');
const { CronJob, CronTime } = require('cron');

const log = require('./logger');
const icon = require('./icon');
const pjson = require('../package.json');
const store = require('./store');

const { app, BrowserWindow, Tray, Menu, shell, ipcMain, Notification } = electron;

app.on('ready', () => {
  const streakerAutoLauncher = new AutoLaunch({
    name: pjson.name
  });

  const tray = new Tray(icon.done);
  let preferencesWindow = null;

  function createPreferencesWindow() {
    preferencesWindow = new BrowserWindow({
      title: `${pjson.name} - Preferences`,
      titleBarStyle: 'hiddenInset',
      width: 500,
      height: 450,
      resizable: false,
      maximizable: false,
      show: true
    });
    preferencesWindow.loadURL(`file://${__dirname}/preferences.html`);
    
    preferencesWindow.on('closed', () => {
      preferencesWindow = null;
    })

    preferencesWindow.on('blur', () => {
      preferencesWindow.close();
    })
    
    preferencesWindow.webContents.toggleDevTools(); // DEV purpose
  }
  createPreferencesWindow(); // DEV purpose

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
      { label: 'Reload', accelerator: 'Cmd+R', click: requestContributionData },
      {
        label: 'Open GitHub Profile...',
        accelerator: 'Cmd+O',
        click: () => shell.openExternal(githubProfileUrl)
      },
      { type: 'separator' },
      {
        label: 'Preferences',
        click: createPreferencesWindow
      },
      { type: 'separator' },
      {
        label: `About ${pjson.name}...`,
        click: () => shell.openExternal(pjson.homepage)
      },
      {
        label: 'Feedback && Support...',
        click: () => shell.openExternal(pjson.bugs.url)
      },
      { type: 'separator' },
      {
        label: `Quit ${pjson.name}`,
        accelerator: 'Cmd+Q',
        click: () => app.quit()
      }
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
      createTrayMenu('Loading...', 'Loading...', 'Loading...')
    );

    setTimeout(requestContributionData, 1000 * 60 * store.get('syncInterval')); // `syncInterval` minutes

    contribution(username)
      .then(data => {
        tray.setContextMenu(
          createTrayMenu(
            data.contributions,
            data.currentStreak,
            data.bestStreak
          )
        );
        tray.setImage(data.currentStreak > 0 ? icon.done : icon.todo);
        log.info(
          `Request successful - username=${username} streak=${
            data.currentStreak
          } today=${data.contributions > 0}`
        );
      })
      .catch(error => {
        tray.setContextMenu(createTrayMenu('Error', 'Error', 'Error'));
        tray.setImage(icon.fail);
        log.error(
          `Request failed - username=${username}) statusCode=${
            error.statusCode
          }`
        );
      });
  }

  function setUsername(event, username) {
    if (username && username !== store.get('username')) {
      store.set('username', username);
      requestContributionData();
      log.info(`Store updated - username=${username}`);
      event.sender.send('usernameSet');
    }
  }

  function setSyncInterval(event, interval) {
    store.set('syncInterval', interval);
    log.info(`Store updated - syncInterval=${interval}`);
    event.sender.send('syncIntervalSet');
  }

  function activateNotifications(event, state) {
    store.set('notification.isEnabled', state);
    log.info(`Store updated - notification.isEnabled=${state}`);
    if (state) {
      job.setTime(new CronTime(`0 ${minutes} ${hours} * * *`));
      job.start();
    } else {
      job.stop();
    }
    event.sender.send('activateNotificationsSet');
  }

  function setNotificationTime(event, data) {
    const { hours, minutes } = data;
    console.log(hours, minutes);
    store.set('notification.time', `${hours}:${minutes}`);
    store.set('notification.hours', hours);
    store.set('notification.minutes', minutes);
    log.info(`Store updated - time=${time} hours=${hours} minutes=${minutes}`);      job.setTime(new CronTime(`0 ${minutes} ${hours} * * *`));
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
          body: 'You haven\'t contributed today'
        }).show()
      }
    }
  });
  
  if (store.get('notification.isEnabled')) {
    job.setTime(new CronTime(`0 ${store.get('notification.minutes')} ${store.get('notification.hours')} * * *`));
    job.start();
  }

  process.on('uncaughtException', (error) => {
      tray.setContextMenu(createTrayMenu('Error', 'Error', 'Error'));
      tray.setImage(icon.fail);
      log.error(error);
  })

  if (process.platform === 'darwin') {
    app.dock.hide();
  }

  app.on('window-all-closed', () => {});
  tray.on('right-click', requestContributionData);
  ipcMain.on('setUsername', setUsername);
  ipcMain.on('setSyncInterval', setSyncInterval);
  ipcMain.on('activateNotifications', activateNotifications);
  ipcMain.on('setNotificationTime', setNotificationTime);

  requestContributionData();
});
