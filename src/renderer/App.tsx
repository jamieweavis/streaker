import React from 'react';
import { Helmet } from 'react-helmet';
import { hot } from 'react-hot-loader';
import { BaseStyles } from '@primer/components';

import { PreferencesForm } from '@renderer/components';

const App = (): JSX.Element => {
  return (
    <BaseStyles>
      <Helmet title="Streaker - Preferences" />
      <PreferencesForm />
      <style
        dangerouslySetInnerHTML={{
          __html: `
            input[type="time"]::-webkit-calendar-picker-indicator {
              display: none;
            }
          `,
        }}
      />
    </BaseStyles>
  );
};

export default hot(module)(App);
