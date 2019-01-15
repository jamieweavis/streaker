const Store = require('electron-store');

module.exports = new Store({
  defaults: {
    username: '',
    userExists: false,
    autoLaunch: false,
    syncInterval: 15,
    notification: {
      isEnabled: false,
      time: '20:00',
    },
  },
});
