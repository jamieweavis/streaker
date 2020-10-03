import Store from 'electron-store';

const store = new Store({
  defaults: {
    username: '',
    launchAtLogin: false,
    syncInterval: 15,
    notificationEnabled: false,
    notificationTime: '20:00',
    iconSet: 'octocat',
  },
});

export default store;
