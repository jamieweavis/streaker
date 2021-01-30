import Store from 'electron-store';

const store = new Store({
  defaults: {
    username: '',
    launchAtLogin: false,
    reminderEnabled: false,
    reminderTime: '20:00',
    iconSet: 'flame',
  },
});

export default store;
