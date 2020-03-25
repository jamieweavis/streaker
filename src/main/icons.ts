import { join } from 'path';

interface Icons {
  done: string;
  todo: string;
  load: string;
  fail: string;
}

let icons: Icons = {
  done: '',
  todo: '',
  load: '',
  fail: '',
};

switch (process.platform) {
  case 'darwin': {
    icons = {
      done: join(__dirname, 'assets/icons/macos/doneTemplate.png'),
      todo: join(__dirname, 'assets/icons/macos/todoTemplate.png'),
      load: join(__dirname, 'assets/icons/macos/loadTemplate.png'),
      fail: join(__dirname, 'assets/icons/macos/failTemplate.png'),
    };
    break;
  }
  case 'win32': {
    icons = {
      done: join(__dirname, 'assets/icons/win/done.ico'),
      todo: join(__dirname, 'assets/icons/win/todo.ico'),
      load: join(__dirname, 'assets/icons/win/load.ico'),
      fail: join(__dirname, 'assets/icons/win/fail.ico'),
    };
    break;
  }
}

export default icons;
