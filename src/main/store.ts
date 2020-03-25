import Store from 'electron-store';

const store = new Store({
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

export default store;
