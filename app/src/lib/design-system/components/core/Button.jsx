import React from 'react';
export function Button({variant='primary',size='md',children,onClick,disabled,icon}){
const pad=size==='sm'?'8px 14px':size==='lg'?'14px 24px':'11px 18px';
const font=size==='sm'?'var(--text-body-sm)':'var(--text-body-md)';
const base={display:'inline-flex',alignItems:'center',gap:8,padding:pad,borderRadius:'var(--radius-md)',font,fontWeight:600,cursor:disabled?'not-allowed':'pointer',border:'1px solid transparent',transition:'background .15s ease, border-color .15s ease',opacity:disabled?0.5:1};
const variants={
primary:{background:'var(--anchor-500)',color:'var(--on-primary)'},
secondary:{background:'var(--surface)',color:'var(--ink)',borderColor:'var(--hairline-strong)'},
ghost:{background:'transparent',color:'var(--ink)'},
clay:{background:'var(--clay-500)',color:'var(--on-clay)'},
danger:{background:'var(--danger)',color:'#fff'}
};
return React.createElement('button',{style:{...base,...variants[variant]},onClick,disabled},icon,children);
}
