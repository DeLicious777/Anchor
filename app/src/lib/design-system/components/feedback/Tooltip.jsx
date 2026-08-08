import React from 'react';
export function Tooltip(props){
var label=props.label;
var children=props.children;
var wrapStyle={position:'relative',display:'inline-block'};
var tipStyle={position:'absolute',bottom:'calc(100% + 6px)',left:'50%',transform:'translateX(-50%)',background:'var(--ink)',color:'#fff',padding:'6px 10px',borderRadius:8,font:'var(--text-body-sm)',whiteSpace:'nowrap',opacity:0,pointerEvents:'none'};
return React.createElement('span',{style:wrapStyle},children,React.createElement('span',{style:tipStyle},label));
}
