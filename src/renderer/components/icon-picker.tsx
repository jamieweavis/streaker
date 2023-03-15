import React from 'react';
import { FieldProps } from 'formik';
import { Text, BorderBox, FilterList } from '@primer/components';

import { iconThemes } from '@renderer/icons';

const IconPicker: React.FC<FieldProps> = ({ field, form }): JSX.Element => {
  const currentIconTheme = iconThemes.find(
    (iconTheme) => iconTheme.name === field.value,
  );

  return (
    <React.Fragment>
      <Text
        fontWeight="bold"
        fontSize="14px"
        as="label"
        style={{ display: 'block' }}
        mt="3"
        mb="2"
      >
        Icon theme
      </Text>
      <FilterList>
        {iconThemes.map(({ name, displayName, icons }) => (
          <FilterList.Item
            key={name}
            mr="3"
            selected={name === field.value}
            onClick={(): void => form.setFieldValue('iconTheme', name)}
          >
            <img
              alt="icon"
              src={icons.contributed}
              style={{
                height: 16,
                marginRight: 10,
                position: 'relative',
                top: 2,
              }}
            />
            {displayName}
          </FilterList.Item>
        ))}
      </FilterList>
      <BorderBox mr="3" mt="3" bg="gray.0">
        <table style={{ width: '100%' }}>
          <tr>
            <td style={{ textAlign: 'center' }}>
              <Text fontWeight="bold" fontSize="14px">
                Pending
              </Text>
            </td>
            <td style={{ textAlign: 'center' }}>
              <Text fontWeight="bold" fontSize="14px">
                Contributed
              </Text>
            </td>
            <td style={{ textAlign: 'center' }}>
              <Text fontWeight="bold" fontSize="14px">
                Streaking
              </Text>
            </td>
          </tr>
          <tr>
            <td style={{ textAlign: 'center' }}>
              <img
                alt="pending icon"
                src={currentIconTheme.icons.pending}
                style={{ height: 16 }}
              />
            </td>
            <td style={{ textAlign: 'center' }}>
              <img
                alt="contributed icon"
                src={currentIconTheme.icons.contributed}
                style={{ height: 16 }}
              />
            </td>
            <td style={{ textAlign: 'center' }}>
              <img
                alt="streaking icon"
                src={currentIconTheme.icons.streaking}
                style={{ height: 16 }}
              />
            </td>
          </tr>
        </table>
      </BorderBox>
    </React.Fragment>
  );
};

export default IconPicker;
