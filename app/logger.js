'use strict'

const log = require('electron-log')
log.transports.console = false
log.transports.file.level = 'info'
log.transports.file.format = '[{y}-{m}-{d} {h}:{i}:{s}] [{level}] {text}'

module.exports = log
