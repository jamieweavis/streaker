import { join } from 'path';

interface IconSet {
  contributed: string;
  error: string;
  loading: string;
  pending: string;
}

interface Icons {
  [key: string]: IconSet;
}

let icons: Icons;

if (process.platform === 'darwin') {
  icons = {
    octocat: {
      contributed: join(
        __dirname,
        'icons/darwin/octocat/contributedTemplate.png',
      ),
      error: join(__dirname, 'icons/darwin/octocat/errorTemplate.png'),
      loading: join(__dirname, 'icons/darwin/octocat/loadingTemplate.png'),
      pending: join(__dirname, 'icons/darwin/octocat/pendingTemplate.png'),
    },
  };
}

if (process.platform === 'win32') {
  icons = {
    octocat: {
      contributed: join(__dirname, 'icons/win32/octocat/contributed.ico'),
      error: join(__dirname, 'icons/win32/octocat/error.ico'),
      loading: join(__dirname, 'icons/win32/octocat/loading.ico'),
      pending: join(__dirname, 'icons/win32/octocat/pending.ico'),
    },
  };
}

export default icons;
