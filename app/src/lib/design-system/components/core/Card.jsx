import React from 'react';
export function Card({children,padding='md',style}){
const pad=padding==='sm'?'var(--space-4)':padding==='lg'?'var(--space-8)':'var(--space-6)';
return React.createElement('div',{style:{background:'var(--surface)',borderRadius:'var(--radius-lg)',boxShadow:'var(--shadow-sm)',padding:pad,...style}},children);
}
