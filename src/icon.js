const path = require('path');

switch (process.platform) {
  case 'darwin': {
    module.exports = {
      done: path.join(__dirname, 'icons/macos/doneTemplate.png'),
      todo: path.join(__dirname, 'icons/macos/todoTemplate.png'),
      load: path.join(__dirname, 'icons/macos/loadTemplate.png'),
      fail: path.join(__dirname, 'icons/macos/failTemplate.png'),
    };
    break;
  }
  case 'win32': {
    module.exports = {
      done: path.join(__dirname, 'icons/win/done.ico'),
      todo: path.join(__dirname, 'icons/win/todo.ico'),
      load: path.join(__dirname, 'icons/win/load.ico'),
      fail: path.join(__dirname, 'icons/win/fail.ico'),
    };
    break;
  }
}
