import { ReactNode, MouseEventHandler } from 'react';
export interface IconButtonProps {
  icon: ReactNode;
  size?: number;
  variant?: 'ghost' | 'filled';
  onClick?: MouseEventHandler;
  'aria-label': string;
}
