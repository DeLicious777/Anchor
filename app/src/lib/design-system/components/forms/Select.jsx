import React from 'react';
export function Select({label,options=[],value,onChange}){
return React.createElement('label',{style:{display:'flex',flexDirection:'column',gap:6,font:'var(--text-body-sm)',color:'var(--ink-soft)'}},
label,
React.createElement('select',{value,onChange,style:{padding:'10px 14px',borderRadius:'var(--radius-md)',border:'1px solid var(--hairline-strong)',background:'var(--surface)',font:'var(--text-body-md)',color:'var(--ink)'}},
options.map((o,i)=>React.createElement('option',{key:i,value:o},o))));
}
