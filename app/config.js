'use strict'

const Config = require('electron-config')

module.exports = new Config({
  defaults: {
    username: '',
    autoLaunch: false
  }
})
