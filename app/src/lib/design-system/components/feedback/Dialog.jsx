import React from 'react';
export function Dialog({open,title,children,onClose}){
if(!open) return null;
return React.createElement('div',{style:{position:'fixed',inset:0,background:'rgba(26,23,18,0.4)',display:'flex',alignItems:'center',justifyContent:'center',zIndex:100}},
React.createElement('div',{style:{background:'var(--surface)',borderRadius:'var(--radius-xl)',padding:'var(--space-8)',minWidth:360,boxShadow:'var(--shadow-lg)'}},
React.createElement('div',{style:{display:'flex',justifyContent:'space-between',alignItems:'center',marginBottom:'var(--space-4)'}},
React.createElement('h3',{style:{font:'var(--text-title-lg)',margin:0,color:'var(--ink)'}},title),
React.createElement('button',{onClick:onClose,style:{border:'none',background:'none',cursor:'pointer',color:'var(--muted)',fontSize:18}},'\u2715')),
children));
}
