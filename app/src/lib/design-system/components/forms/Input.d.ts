import { ChangeEventHandler } from 'react';
export interface InputProps {
  label?: string;
  placeholder?: string;
  value?: string;
  onChange?: ChangeEventHandler;
  type?: string;
}
