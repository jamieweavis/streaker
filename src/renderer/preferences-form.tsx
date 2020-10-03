import { remote, ipcRenderer } from 'electron';
import React from 'react';
import { MarkGithubIcon, ClockIcon } from '@primer/octicons-react';
import { Formik, Field, Form } from 'formik';
import {
  TextInput,
  Text,
  Avatar,
  Button,
  ButtonPrimary,
  Flex,
  Dropdown,
  Box,
  FilterList,
} from '@primer/components';

import { icons, IconSets } from './icons';
import { PreferencesSavedValues } from '../common/types';
import store from '../common/store';

const closeWindow = (): void => {
  const window = remote.getCurrentWindow();
  window.close();
};

const PreferencesForm = (): JSX.Element => (
  <Formik
    initialValues={{
      username: store.get('username'),
      pollInterval: store.get('pollInterval'),
      launchAtLogin: store.get('launchAtLogin'),
      reminderEnabled: store.get('reminderEnabled'),
      reminderTime: store.get('reminderTime'),
      iconSet: store.get('iconSet'),
    }}
    onSubmit={async (values: PreferencesSavedValues): Promise<void> => {
      await ipcRenderer.send('preferences-saved', values);
      closeWindow();
    }}
  >
    <Form style={{ paddingLeft: 15 }}>
      <Text
        fontWeight="bold"
        fontSize="14px"
        as="label"
        htmlFor="github-username"
        style={{ display: 'block' }}
        mt="2"
        mb="2"
      >
        GitHub username
      </Text>
      <Field
        name="username"
        as={(props): JSX.Element => (
          <TextInput
            id="github-username"
            icon={MarkGithubIcon}
            placeholder="GitHub Username"
            width={175}
            {...props}
          />
        )}
      />
      <Field
        name="username"
        render={({ field }): JSX.Element => (
          <Avatar
            src={`https://avatars.githubusercontent.com/${field?.value || '_'}`}
            square
            ml="2"
          />
        )}
      />
      <Text
        fontWeight="bold"
        fontSize="14px"
        as="label"
        htmlFor="sync-interval"
        style={{ display: 'block' }}
        mt="3"
        mb="2"
      >
        Sync interval
      </Text>
      <Text fontSize="14px">Sync every </Text>
      <Field
        name="pollInterval"
        as={(props): JSX.Element => (
          <TextInput
            id="sync-interval"
            placeholder="15"
            width={70}
            type="number"
            {...props}
          />
        )}
      />
      <Text fontSize="14px"> minutes</Text>
      <Text
        fontWeight="bold"
        fontSize="14px"
        as="label"
        htmlFor="launch-at-login"
        style={{ display: 'block' }}
        mt="3"
        mb="2"
      >
        <Field id="launch-at-login" type="checkbox" name="launchAtLogin" />
        <Text> Launch at login</Text>
      </Text>
      <Text
        fontWeight="bold"
        fontSize="14px"
        as="label"
        htmlFor="notification-enabled"
        style={{ display: 'block' }}
        mt="3"
      >
        <Field
          id="notification-enabled"
          type="checkbox"
          name="reminderEnabled"
        />
        <Text> Enable reminder notification</Text>
      </Text>
      <Text color="grey" fontSize="12px" as="p">
        Get a reminder so you don&apos;t lose your streak! 🔥
      </Text>
      <Text
        fontWeight="bold"
        fontSize="14px"
        as="label"
        htmlFor="reminder-time"
        style={{ display: 'block' }}
        mt="3"
        mb="2"
      >
        Reminder time
      </Text>
      <Text fontSize="14px">Remind me at </Text>
      <Field
        name="reminderTime"
        render={(props): JSX.Element => {
          console.log(props);
          return (
            <TextInput
              id="reminder-time"
              icon={ClockIcon}
              placeholder="Reminder Time"
              type="time"
              onChange={(e): void =>
                props.form.setFieldValue('reminderTime', e.target.value)
              }
              value={props.field.value}
              disabled={!props.form.values.reminderEnabled}
              {...props}
            />
          );
        }}
      />
      <Text
        fontWeight="bold"
        fontSize="14px"
        as="label"
        style={{ display: 'block' }}
        mt="3"
        mb="2"
      >
        Icon set
      </Text>
      <Field
        name="iconSet"
        render={({ field, form }): JSX.Element => {
          return (
            <React.Fragment>
              <FilterList>
                {Object.keys(IconSets).map((iconSet) => (
                  <FilterList.Item
                    key={iconSet}
                    href="#bar"
                    mr="3"
                    selected={iconSet === store.get('iconSet')}
                    onClick={(): void => {
                      form.setFieldValue('iconSet', iconSet);
                    }}
                  >
                    <img
                      src={icons[field.value].contributed}
                      style={{
                        height: 16,
                        marginRight: 10,
                        position: 'relative',
                        top: 2,
                      }}
                    />
                    {IconSets[iconSet]}
                  </FilterList.Item>
                ))}
              </FilterList>
              <Box ml="3" pt="2">
                <img
                  src={icons[field.value].contributed}
                  style={{ height: 20, marginRight: 10 }}
                />
                <img
                  src={icons[field.value].error}
                  style={{ height: 20, marginRight: 10 }}
                />
                <img
                  src={icons[field.value].loading}
                  style={{ height: 20, marginRight: 10 }}
                />
                <img
                  src={icons[field.value].pending}
                  style={{ height: 20, marginRight: 10 }}
                />
              </Box>
            </React.Fragment>
          );
        }}
      />
      <Flex p="3">
        <Button onClick={closeWindow} ml="auto">
          Cancel
        </Button>
        <ButtonPrimary type="submit" ml="2">
          Save
        </ButtonPrimary>
      </Flex>
    </Form>
  </Formik>
);

export default PreferencesForm;
