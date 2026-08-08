import React from 'react';
export function Checkbox({label,checked,onChange}){
return React.createElement('label',{style:{display:'flex',alignItems:'center',gap:8,font:'var(--text-body-md)',color:'var(--ink)',cursor:'pointer'}},
React.createElement('span',{style:{width:18,height:18,borderRadius:6,border:checked?'none':'1.5px solid var(--hairline-strong)',background:checked?'var(--anchor-500)':'var(--surface)',display:'inline-flex',alignItems:'center',justifyContent:'center'}},
checked?React.createElement('svg',{width:11,height:11,viewBox:'0 0 16 16'},React.createElement('path',{d:'M2 8l4 4 8-8',stroke:'#fff',strokeWidth:2,fill:'none'})):null),
React.createElement('input',{type:'checkbox',checked,onChange,style:{display:'none'}}),
label);
}
