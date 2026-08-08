import { ChangeEventHandler } from 'react';
export interface SelectProps {
  label?: string;
  options: string[];
  value?: string;
  onChange?: ChangeEventHandler;
}
