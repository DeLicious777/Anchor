import React from 'react';
export function Toast({message,variant='default'}){
const bg=variant==='success'?'var(--success-100)':variant==='danger'?'var(--danger-100)':'var(--surface)';
const fg=variant==='success'?'var(--success)':variant==='danger'?'var(--danger)':'var(--ink)';
return React.createElement('div',{style:{display:'inline-flex',alignItems:'center',gap:8,padding:'12px 18px',borderRadius:'var(--radius-md)',background:bg,color:fg,font:'var(--text-body-sm)',boxShadow:'var(--shadow-md)'}},message);
}
