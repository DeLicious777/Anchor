import { MouseEventHandler } from 'react';
export interface SwitchProps {
  checked?: boolean;
  onChange?: MouseEventHandler;
  label?: string;
}
