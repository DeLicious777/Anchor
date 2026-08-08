import React from 'react';
export function Switch({checked,onChange,label}){
return React.createElement('label',{style:{display:'flex',alignItems:'center',gap:10,cursor:'pointer',font:'var(--text-body-md)',color:'var(--ink)'}},
React.createElement('span',{onClick:onChange,style:{width:40,height:24,borderRadius:'var(--radius-pill)',background:checked?'var(--anchor-500)':'var(--hairline-strong)',position:'relative',transition:'background .15s ease',display:'inline-block'}},
React.createElement('span',{style:{position:'absolute',top:3,left:checked?19:3,width:18,height:18,borderRadius:'50%',background:'#fff',transition:'left .15s ease',boxShadow:'var(--shadow-sm)'}})),
label);
}
