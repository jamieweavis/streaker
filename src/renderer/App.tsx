import React from 'react';
import { Helmet } from 'react-helmet';
import { hot } from 'react-hot-loader';
import { BaseStyles } from '@primer/components';

import PreferencesForm from './preferences-form';

const App = (): JSX.Element => {
  return (
    <BaseStyles>
      <Helmet title="Streaker - Preferences" />
      <PreferencesForm />
    </BaseStyles>
  );
};

export default hot(module)(App);
