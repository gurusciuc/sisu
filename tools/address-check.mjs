import assert from 'node:assert/strict';
import { fixture } from './fixture.mjs';
import { spawn, execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { mkdtemp, mkdir, writeFile } from 'node:fs/promises';
import { setTimeout as sleep } from 'node:timers/promises';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
const [exe,dir]=process.argv.slice(2);
if(!exe||!dir)throw Error('Expected browser and results directory');
const out=path.resolve(dir), home=await mkdtemp(path.join(tmpdir(),'sisu-address-')), f=await fixture();
await mkdir(out,{recursive:true});
const script=fileURLToPath(new URL('./control.ps1',import.meta.url));
let child;
const action=async(kind,value)=> (await promisify(execFile)('powershell.exe',['-NoProfile','-ExecutionPolicy','Bypass','-File',script,'-BrowserPid',String(child.pid),'-Action',kind,...(value?['-Value',value]:[])],{windowsHide:true,timeout:20000})).stdout.trim();
async function until(test){for(let i=0;i<120;i++){if(test())return;await sleep(80);}throw Error('Timed out');}
async function start(run,flags=[]){
 child=spawn(path.resolve(exe),[...flags,'--url='+f.url+'/?run='+run],{env:{...process.env,BROWSER_TEST_HOME:home},stdio:'ignore'});
 await until(()=>f.states.has(run));
}
async function close(){if(child?.exitCode===null){await action('close');await until(()=>child.exitCode!==null);assert.equal(child.exitCode,0);}}
try{
 await start('address-first');
 await action('navigate',f.url+'/second?run=address-second');await until(()=>f.states.has('address-second'));
 await action('type','address-first');
 const matches=JSON.parse(await action('suggestions'));
 assert(matches.includes(f.url+'/?run=address-first'),JSON.stringify(matches));
 await action('addressKey','down');await action('addressKey','enter');
 assert.equal(await action('address'),f.url+'/?run=address-first');
 await close();await start('address-restart');
 await action('type','address-second');assert(JSON.parse(await action('suggestions')).includes(f.url+'/second?run=address-second'));
 await action('type','some words for a search');
 assert.equal(JSON.parse(await action('suggestions'))[0],'Search for “some words for a search”');
 const before=f.hits.length;await sleep(300);
 assert.equal(f.hits.length,before,'Typing contacted the fixture server');
 await close();await start('address-private',['--private']);
 await action('type','address-second');assert.equal(JSON.parse(await action('suggestions')).length,1,'Ordinary history appeared in private suggestions');
 await close();
 await writeFile(path.join(out,'address.json'),JSON.stringify({localHistory:true,keyboardSelection:true,persistentRecent:true,searchRow:true,noFixtureRequestsWhileTyping:true,privateSeparation:true},null,2));
}finally{await close();f.server.closeAllConnections();f.server.close();}
