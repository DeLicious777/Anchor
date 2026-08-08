import React from 'react';
const palette={amber:['tag-amber','tag-amber-bg'],coral:['tag-coral','tag-coral-bg'],teal:['tag-teal','tag-teal-bg'],indigo:['tag-indigo','tag-indigo-bg'],moss:['tag-moss','tag-moss-bg'],plum:['tag-plum','tag-plum-bg'],sky:['tag-sky','tag-sky-bg'],clay:['tag-clay','tag-clay-bg']};
export function Tag({children,color='indigo'}){
const [fg,bg]=palette[color]||palette.indigo;
return React.createElement('span',{style:{display:'inline-flex',alignItems:'center',padding:'4px 12px',borderRadius:'var(--radius-pill)',font:'var(--text-label)',letterSpacing:'var(--tracking-label)',color:`var(--${fg})`,background:`var(--${bg})`}},children);
}
