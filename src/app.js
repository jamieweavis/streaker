'use strict'

const log = require('./logger')
const icon = require('./icon')
const pjson = require('../package.json')
const store = require('./store')
const AutoLaunch = require('auto-launch')
const contribution = require("contribution");

const electron = require('electron')
const { app, BrowserWindow, Tray, Menu, shell, ipcMain } = electron

app.on('ready', () => {
  const streakerAutoLauncher = new AutoLaunch({
    name: pjson.name,
    path: `/Applications/${pjson.name}.app`
  })

  let tray = new Tray(icon.done)
  let usernameWindow = null

  function requestContributionData () {
    tray.setImage(icon.load)
    tray.setContextMenu(createTrayMenu('Loading...'))

    let username = store.get('username')

    contribution(username).then(data => {
      tray.setContextMenu(createTrayMenu(`Streak: ${data.streak}`))
      tray.setImage(data.streak > 0 ? icon.done : icon.todo)
      log.info(`Request successful - username=${username} streak=${data.streak} today=${data.contribution > 0}`);
    }).catch((error) => {
      tray.setContextMenu(createTrayMenu('Failed to get streak'))
      tray.setImage(icon.fail)
      log.error(`Request failed - username=${username}) statusCode=${error.statusCode}`)
    })
  }

  function createTrayMenu (displayLabel) {
    let username = store.get('username') || 'username not set'
    let githubProfileUrl = `https://github.com/${username}`
    let menuTemplate = [
      { label: `${displayLabel} (${username})`, enabled: false },
      { type: 'separator' },
      { label: 'Reload', accelerator: 'Cmd+R', click: requestContributionData },
      { label: 'Open GitHub Profile...', accelerator: 'Cmd+O', click: () => { shell.openExternal(githubProfileUrl) } },
      { type: 'separator' },
      { label: 'Set GitHub Username...', accelerator: 'Cmd+S', click: createUsernameWindow },
      {
        label: 'Preferences',
        submenu: [{
          label: `Launch ${pjson.name} at login`,
          type: 'checkbox',
          checked: store.get('autoLaunch'),
          click: (checkbox) => {
            store.set('autoLaunch', checkbox.checked)
            checkbox.checked ? streakerAutoLauncher.enable() : streakerAutoLauncher.disable()
            log.info(`Store updated - autoLaunch=${checkbox.checked}`)
          }
        }]
      },
      { type: 'separator' },
      { label: `About ${pjson.name}...`, click: () => { shell.openExternal(pjson.homepage) } },
      { label: 'Feedback && Support...', click: () => { shell.openExternal(pjson.bugs.url) } },
      { type: 'separator' },
      { label: `Quit ${pjson.name}`, accelerator: 'Cmd+Q', click: () => { app.quit() } }
    ]
    return Menu.buildFromTemplate(menuTemplate)
  }

  function createUsernameWindow () {
    if (usernameWindow) return usernameWindow.focus()
    usernameWindow = new BrowserWindow({
      title: `${pjson.name} - Set GitHub Username`,
      frame: false,
      width: 270,
      height: 60,
      resizable: false,
      maximizable: false,
      show: false
    })
    usernameWindow.loadURL(`file://${__dirname}/username.html`)
    usernameWindow.once('ready-to-show', () => {
      let screen = electron.screen.getDisplayNearestPoint(electron.screen.getCursorScreenPoint())
      usernameWindow.setPosition(
        Math.floor(screen.bounds.x + (screen.size.width / 2) - (usernameWindow.getSize()[0] / 2)),
        Math.floor(screen.bounds.y + (screen.size.height / 2) - (usernameWindow.getSize()[1] / 2))
      )
      usernameWindow.show()
    })
    usernameWindow.on('closed', () => { usernameWindow = null })
    usernameWindow.on('blur', () => { usernameWindow.close() })
  }

  function setUsername (event, username) {
    usernameWindow.close()
    if (username && username !== store.get('username')) {
      store.set('username', username)
      requestContributionData()
      log.info(`Store updated - username=${username}`)
    }
  }

  app.dock.hide()
  app.on('window-all-closed', () => {})
  tray.on('right-click', requestContributionData)
  ipcMain.on('setUsername', setUsername)

  requestContributionData()
  setInterval(requestContributionData, 1000 * 60 * 15) // 15 Minutes
})
