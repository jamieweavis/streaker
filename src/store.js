const Store = require('electron-store');

module.exports = new Store({
  defaults: {
    username: '',
    autoLaunch: false,
    syncInterval: 15,
    notification: {
      isEnabled: false,
      hours: '20',
      minutes: '00',
    },
  },
});
