import { ReactNode, MouseEventHandler } from 'react';
export interface ButtonProps {
  variant?: 'primary' | 'secondary' | 'ghost' | 'clay' | 'danger';
  size?: 'sm' | 'md' | 'lg';
  children: ReactNode;
  onClick?: MouseEventHandler;
  disabled?: boolean;
  icon?: ReactNode;
}
