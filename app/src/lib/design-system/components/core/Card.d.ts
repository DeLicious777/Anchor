import { ReactNode, CSSProperties } from 'react';
export interface CardProps {
  children: ReactNode;
  padding?: 'sm' | 'md' | 'lg';
  style?: CSSProperties;
}
