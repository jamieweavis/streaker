import React from 'react';
import { Formik, Field, Form } from 'formik';
import { remote, ipcRenderer } from 'electron';
import { MarkGithubIcon, ClockIcon } from '@primer/octicons-react';
import {
  TextInput,
  Text,
  Avatar,
  Button,
  ButtonPrimary,
  Flex,
} from '@primer/components';

import { PreferencesSavedValues } from '@common/types';
import { IconPicker } from '@renderer/components';
import store from '@common/store';

const closeWindow = (): void => {
  const window = remote.getCurrentWindow();
  window.close();
};

const PreferencesForm = (): JSX.Element => (
  <Formik
    initialValues={{
      username: store.get('username'),
      launchAtLogin: store.get('launchAtLogin'),
      reminderEnabled: store.get('reminderEnabled'),
      reminderTime: store.get('reminderTime'),
      iconTheme: store.get('iconTheme'),
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
        // @ts-ignore
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
        // @ts-ignore
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
        // @ts-ignore
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
        // @ts-ignore
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
        render={({ form, field, ...rest }): JSX.Element => (
          <TextInput
            id="reminder-time"
            icon={ClockIcon}
            placeholder="Reminder Time"
            type="time"
            onChange={(e): void =>
              form.setFieldValue('reminderTime', e.target.value)
            }
            value={field.value}
            disabled={!form.values.reminderEnabled}
            pr="2"
            {...rest}
          />
        )}
      />
      <Field name="iconTheme" component={IconPicker} />
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
