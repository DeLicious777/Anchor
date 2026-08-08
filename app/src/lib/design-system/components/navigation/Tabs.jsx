import React from 'react';
export function Tabs({items=[],active,onChange}){
return React.createElement('div',{style:{display:'flex',gap:4,borderBottom:'1px solid var(--hairline)'}},
items.map((it,i)=>React.createElement('button',{key:i,onClick:()=>onChange&&onChange(it),style:{padding:'10px 16px',background:'none',border:'none',borderBottom:it===active?'2px solid var(--anchor-500)':'2px solid transparent',color:it===active?'var(--ink)':'var(--muted)',font:'var(--text-body-md)',fontWeight:it===active?600:400,cursor:'pointer'}},it)));
}
