'use strict'

const Config = require('electron-config')

module.exports = new Config({
  defaults: {
    githubUsername: '',
    autoLaunch: false,
    refreshInterval: 900000 // 15 Minutes
  }
})
