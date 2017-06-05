'use strict'

const Config = require('electron-config')

module.exports = new Config({
  defaults: {
    githubUsername: '',
    autoLaunch: false,
    requestInterval: 1000 * 60 * 10 // 10 Minutes
  }
})
