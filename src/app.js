const electron = require('electron');
const AutoLaunch = require('auto-launch');
const contribution = require('contribution');
const CronJob = require('cron').CronJob;

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
  let usernameWindow = null;
  let notificationWindow = null;

  function createUsernameWindow() {
    if (usernameWindow) {
      usernameWindow.focus();
      return;
    }
    usernameWindow = new BrowserWindow({
      title: `${pjson.name} - Set GitHub Username`,
      frame: false,
      width: 270,
      height: 60,
      resizable: false,
      maximizable: false,
      show: false
    });
    usernameWindow.loadURL(`file://${__dirname}/username.html`);
    usernameWindow.once('ready-to-show', () => {
      const screen = electron.screen.getDisplayNearestPoint(
        electron.screen.getCursorScreenPoint()
      );
      usernameWindow.setPosition(
        Math.floor(
          screen.bounds.x +
            screen.size.width / 2 -
            usernameWindow.getSize()[0] / 2
        ),
        Math.floor(
          screen.bounds.y +
            screen.size.height / 2 -
            usernameWindow.getSize()[1] / 2
        )
      );
      usernameWindow.show();
    });
    usernameWindow.on('closed', () => {
      usernameWindow = null;
    });
    usernameWindow.on('blur', () => {
      usernameWindow.close();
    });
  }

  function createNotificationWindow() {
    if (notificationWindow) {
      notificationWindow.focus();
      return;
    }
    notificationWindow = new BrowserWindow({
      title: `${pjson.name} - Set GitHub Username`,
      frame: false,
      width: 270,
      height: 60,
      resizable: false,
      maximizable: false,
      show: false
    });
    notificationWindow.loadURL(`file://${__dirname}/notification.html`);
    notificationWindow.once('ready-to-show', () => {
      const screen = electron.screen.getDisplayNearestPoint(
        electron.screen.getCursorScreenPoint()
      );
      notificationWindow.setPosition(
        Math.floor(
          screen.bounds.x +
            screen.size.width / 2 -
            notificationWindow.getSize()[0] / 2
        ),
        Math.floor(
          screen.bounds.y +
            screen.size.height / 2 -
            notificationWindow.getSize()[1] / 2
        )
      );
      notificationWindow.show();
    });
    notificationWindow.on('closed', () => {
      notificationWindow = null;
    });
    notificationWindow.on('blur', () => {
      notificationWindow.close();
    });
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
      { label: 'Reload', accelerator: 'Cmd+R', click: requestContributionData },
      {
        label: 'Open GitHub Profile...',
        accelerator: 'Cmd+O',
        click: () => shell.openExternal(githubProfileUrl)
      },
      { type: 'separator' },
      {
        label: 'Set GitHub Username...',
        accelerator: 'Cmd+S',
        click: createUsernameWindow
      },
      {
        label: 'Preferences',
        submenu: [
          {
            label: `Launch ${pjson.name} at login`,
            type: 'checkbox',
            checked: store.get('autoLaunch'),
            click: checkbox => {
              store.set('autoLaunch', checkbox.checked);
              if (checkbox.checked) {
                streakerAutoLauncher.enable();
              } else {
                streakerAutoLauncher.disable();
              }
              log.info(`Store updated - autoLaunch=${checkbox.checked}`);
            }
          },
          {
            label: 'Activate notifications',
            type: 'checkbox',
            checked: store.get('notification.isEnabled'),
            click: checkbox => {
              store.set('notification.isEnabled', checkbox.checked);
              if (checkbox.checked) {
                job.start();
              } else {
                job.stop();
              }
              log.info(`Store updated - notification=${checkbox.checked}`);
            }
          },
          {
            label: 'Set notification time',
            click: createNotificationWindow
          }
        ]
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
      createUsernameWindow();
      return;
    }

    tray.setImage(icon.load);
    tray.setContextMenu(
      createTrayMenu('Loading...', 'Loading...', 'Loading...')
    );

    setTimeout(requestContributionData, 1000 * 60 * 60); // 15 Minutes

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
    usernameWindow.close();
    if (username && username !== store.get('username')) {
      store.set('username', username);
      requestContributionData();
      log.info(`Store updated - username=${username}`);
    }
  }

  function setNotificationTime(event, time) {
    notificationWindow.close();
    if (time && time !== store.get('notification.time')) {
      let hours = time.split(':')[0];
      let minutes = time.split(':')[1];
      store.set('notification.time', time);
      store.set('notification.hours', hours);
      store.set('notification.minutes', minutes)
    }
  }

  const job = new CronJob({
    cronTime: `0 0 20 * * *`,
    onTick: async function() {
      const data = await contribution(store.get('username'));
      if (data.currentStreak === 0 && Notification.isSupported()) {
        new Notification({
          title: 'Streaker',
          body: 'You didn\'t contribute today',
          icon: icon.icon
        }).show()
      }
    }
  });
  
  if (store.get('notification.isEnabled')) {
    job.start();
  } else {
    job.stop();
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
  ipcMain.on('setNotificationTime', setNotificationTime);

  requestContributionData();
});
