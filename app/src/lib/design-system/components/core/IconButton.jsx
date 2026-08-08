import React from 'react';
export function IconButton({icon,size=36,variant='ghost',onClick,'aria-label':ariaLabel}){
const bg=variant==='filled'?'var(--surface-sunken)':'transparent';
return React.createElement('button',{'aria-label':ariaLabel,onClick,style:{width:size,height:size,borderRadius:'var(--radius-pill)',background:bg,border:'none',display:'inline-flex',alignItems:'center',justifyContent:'center',cursor:'pointer',color:'var(--ink-soft)'}},icon);
}
