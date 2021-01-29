import Store from 'electron-store';

const store = new Store({
  defaults: {
    username: '',
    launchAtLogin: false,
    pollInterval: 15,
    reminderEnabled: false,
    reminderTime: '20:00',
    iconSet: 'octocat',
  },
});

export default store;
