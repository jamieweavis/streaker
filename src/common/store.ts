import Store from 'electron-store';

const store = new Store({
  defaults: {
    username: '',
    userExists: false,
    launchAtLogin: false,
    syncInterval: 15,
    notificationEnabled: false,
    notificationTime: '20:00',
  },
});

export default store;
