import React, { Fragment } from 'react';
import { hot } from 'react-hot-loader';
import { Avatar, BranchName } from '@primer/components';

import store from '../common/store';

const App = (): JSX.Element => (
  <Fragment>
    <Avatar
      square
      src={`https://avatars.githubusercontent.com/${store.get('username')}`}
    />
    <BranchName>{store.get('username')}</BranchName>
  </Fragment>
);

export default hot(module)(App);
