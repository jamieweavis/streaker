const Store = require('electron-store');

module.exports = new Store({
  defaults: {
    username: '',
    autoLaunch: false,
    notification: {
      isEnabled: false,
      time: '20:00',
      hours: '20',
      minutes: '00'
    }
  }
});
