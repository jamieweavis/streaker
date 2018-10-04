const Store = require('electron-store');

module.exports = new Store({
  defaults: {
    username: '',
    autoLaunch: false,
    notification: false,
    contributedToday: false
  }
});
