'use strict'

const path = require('path')
const pjson = require('./package.json')
const config = require('./app/config')
const request = require('request-promise-native')
const DOMParser = require('dom-parser')
const AutoLaunch = require('auto-launch')

const electron = require('electron')
const { app, BrowserWindow, Tray, Menu, shell, ipcMain } = electron

let iconDone = path.join(__dirname, 'app', 'doneTemplate.png')
let iconTodo = path.join(__dirname, 'app', 'todoTemplate.png')
let iconWarning = path.join(__dirname, 'app', 'todoTemplate.png') // @TODO: Create warning icon
let iconLoading = path.join(__dirname, 'app', 'todoTemplate.png') // @TODO: Create loading icon

app.on('ready', () => {
  const parser = new DOMParser()
  const streakerAutoLauncher = new AutoLaunch({
    name: app.getName(),
    path: `/Applications/${pjson.name}.app`
  })

  let tray = new Tray(iconDone)
  let usernameWindow = null

  function requestContributionData () {
    tray.setImage(iconLoading)
    tray.setContextMenu(createTrayMenu('Loading...'))

    let contributionUrl = `https://github.com/users/${config.get('githubUsername')}/contributions`
    let contributionStreak = 0
    let contributedToday = false

    request(contributionUrl).then((htmlString) => {
      const days = parser.parseFromString(htmlString, 'text/html').getElementsByClassName('day')
      days.forEach((day, index) => {
        if (day.getAttribute('data-count') > 0) {
          contributionStreak++
          contributedToday = true
        } else {
          if (days.length - 1 !== index) contributionStreak = 0
          contributedToday = false
        }
      })
      tray.setContextMenu(createTrayMenu(`Streak: ${contributionStreak}`))
      tray.setImage(contributedToday ? iconDone : iconTodo)
    }).catch((error) => {
      tray.setContextMenu(createTrayMenu('Failed to get streak'))
      tray.setImage(iconWarning)
    })
  }

  function createTrayMenu (displayLabel) {
    let username = config.get('githubUsername') || 'username not set'
    let menuTemplate = [
      { label: `${displayLabel} (${username})`, enabled: false },
      { type: 'separator' },
      { label: 'Refresh', accelerator: 'Cmd+R', click: requestContributionData },
      { label: 'Set GitHub Username...', click: createUsernameWindow },
      {
        label: 'Preferences',
        submenu: [{
          label: `Launch ${pjson.name} at login`,
          type: 'checkbox',
          checked: config.get('autoLaunch'),
          click: (checkbox) => {
            config.set('autoLaunch', checkbox.checked)
            checkbox.checked ? streakerAutoLauncher.enable() : streakerAutoLauncher.disable()
          }
        }]
      },
      { type: 'separator' },
      { label: `About ${pjson.name}`, click: () => { shell.openExternal(pjson.homepage) } },
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
      width: 265,
      height: 60,
      resizable: false,
      maximizable: false,
      show: false
    })
    usernameWindow.loadURL(`file://${__dirname}/app/username.html`)
    usernameWindow.once('ready-to-show', () => { usernameWindow.show() })
    usernameWindow.on('closed', () => { usernameWindow = null })
  }

  app.dock.hide()
  app.on('window-all-closed', () => {})

  tray.on('right-click', requestContributionData)

  ipcMain.on('setUsername', (event, username) => {
    usernameWindow.close()
    if (username && username !== config.get('githubUsername')) {
      config.set('githubUsername', username)
      requestContributionData()
    }
  })

  requestContributionData()
  setInterval(requestContributionData, config.get('requestInterval'))
})
