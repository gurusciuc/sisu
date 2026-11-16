import assert from 'node:assert/strict';
import { fixture } from './fixture.mjs';
import { spawn,execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { mkdtemp,mkdir,writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { setTimeout as sleep } from 'node:timers/promises';
const [exe,dir]=process.argv.slice(2);
if(!exe||!dir)throw Error('Expected staged executable and evidence directory');
const f=await fixture(),home=await mkdtemp(path.join(tmpdir(),'sisu-keys-'));
await mkdir(dir,{recursive:true});
const child=spawn(path.resolve(exe),['--url='+f.url+'/?run=tab-anchor'],{env:{...process.env,BROWSER_TEST_HOME:home},stdio:'ignore'});
const ps=(script,action,value)=>promisify(execFile)('powershell.exe',['-NoProfile','-ExecutionPolicy','Bypass','-File',fileURLToPath(new URL(script,import.meta.url)),'-BrowserPid',String(child.pid),'-Action',action,...(value?['-Value',value]:[])],{windowsHide:true,timeout:20000});
async function waitFor(fn){for(let i=0;i<150;i++){if(fn())return;await sleep(80);}throw Error('Timed out waiting for key-driven change');}
try{
 await waitFor(()=>f.states.has('tab-anchor'));
 const original=f.states.get('tab-anchor');
 for(const layout of ['top','left']){
  if(layout==='left')await ps('./control.ps1','button','Change tab placement');
  for(let i=0;i<3;i++){
   await ps('./window.ps1','keys','^t');
   assert.equal((await ps('./control.ps1','address')).stdout.trim(),'');
   await ps('./window.ps1','keys','^w');
   await sleep(150);
   assert.equal((await ps('./control.ps1','address')).stdout.trim(),f.url+'/?run=tab-anchor');
   assert.equal(f.states.get('tab-anchor').documentId,original.documentId);
   assert.equal(f.states.get('tab-anchor').draft,original.draft);
  }
 }
 await writeFile(path.join(dir,'keyboard.json'),JSON.stringify({controlT:true,controlW:true,cycles:6,layouts:['top','left'],anchorPreserved:true},null,2));
}finally{
 if(child.exitCode===null)await ps('./control.ps1','close');
 f.server.closeAllConnections();f.server.close();
}
