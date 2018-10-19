const path = require('path');

if (process.platform === 'win32') {
  module.exports = {
    streaker: path.join(__dirname, 'icons/win-icon.png'),
    done: path.join(__dirname, 'icons/win/done.ico'),
    todo: path.join(__dirname, 'icons/win/todo.ico'),
    load: path.join(__dirname, 'icons/win/load.ico'),
    fail: path.join(__dirname, 'icons/win/fail.ico')
  }
} else {
  module.exports = {
    streaker: path.join(__dirname, 'icons/icon.png'),
    done: path.join(__dirname, 'icons/macos/doneTemplate.png'),
    todo: path.join(__dirname, 'icons/macos/todoTemplate.png'),
    load: path.join(__dirname, 'icons/macos/loadTemplate.png'),
    fail: path.join(__dirname, 'icons/macos/failTemplate.png')
  }
}
