const path = require('path');

if (process.platform == 'win32') {
  module.exports = {
    done: path.join(__dirname, 'icons/win/doneTemplate.png'),
    todo: path.join(__dirname, 'icons/win/todoTemplate.png'),
    load: path.join(__dirname, 'icons/win/loadTemplate.png'),
    fail: path.join(__dirname, 'icons/win/failTemplate.png')
  }
} else {
  module.exports = {
    done: path.join(__dirname, 'icons/macos/doneTemplate.png'),
    todo: path.join(__dirname, 'icons/macos/todoTemplate.png'),
    load: path.join(__dirname, 'icons/macos/loadTemplate.png'),
    fail: path.join(__dirname, 'icons/macos/failTemplate.png')
  }
}
