import Store from 'electron-store';

export const store = new Store({
  defaults: {
    username: '',
    launchAtLogin: false,
    reminderEnabled: false,
    reminderTime: '20:00',
    iconTheme: 'flame',
  },
});
