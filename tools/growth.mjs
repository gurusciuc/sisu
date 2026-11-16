import assert from 'node:assert/strict';
import { spawn, execFile } from 'node:child_process';
import { mkdtemp, mkdir, readFile, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { promisify } from 'node:util';
import { setTimeout as sleep } from 'node:timers/promises';
import { fixture } from './fixture.mjs';
const [exe,dir]=process.argv.slice(2);
if(!exe||!dir)throw Error('Expected staged browser and evidence directory');
const out=path.resolve(dir);await mkdir(out,{recursive:true});
const root=fileURLToPath(new URL('./',import.meta.url)), f=await fixture();
const home=await mkdtemp(path.join(tmpdir(),'desktop-growth-'));
const child=spawn(path.resolve(exe),['--url='+f.url+'/?run=tab-anchor'],{env:{...process.env,BROWSER_TEST_HOME:home},stdio:'ignore'});
const ps=(script,...args)=>promisify(execFile)('powershell.exe',['-NoProfile','-ExecutionPolicy','Bypass','-File',path.join(root,script),'-BrowserPid',String(child.pid),...args],{windowsHide:true,timeout:30000});
const action=(kind,value)=>ps('control.ps1','-Action',kind,...(value?['-Value',value]:[]));
async function until(fn){for(let i=0;i<130;i++){if(fn())return;await sleep(80);}throw Error('Fixture did not reach expected state');}
const checkpoints=[];
async function measure(cycles){
  await sleep(1200);
  const file=path.join(out,'growth-'+cycles+'.json');
  await ps('sample-windows.ps1','-Samples','1','-OutputFile',file);
  const raw=JSON.parse(await readFile(file,'utf8')), row=Array.isArray(raw)?raw[0]:raw;
  checkpoints.push({cycles,processes:row.processes.length,rendererCount:row.processes.filter(p=>p.role==='renderer').length,
    privateWorkingSetMiB:row.processes.reduce((n,p)=>n+p.workingSetPrivateBytes,0)/1048576,
    privateCommitMiB:row.processes.reduce((n,p)=>n+p.privateCommitBytes,0)/1048576});
}
try{
  await until(()=>f.states.has('tab-anchor'));const original=f.states.get('tab-anchor');
  await measure(0);
  for(let i=1;i<=12;i++){
    const run='cycle-'+i;
    await action('button','New tab');await action('navigate',f.url+'/?run='+run);
    await until(()=>f.states.has(run));await action('button','Close active tab');
    assert.equal((await action('address')).stdout.trim(),f.url+'/?run=tab-anchor');
    assert.equal(f.states.get('tab-anchor').documentId,original.documentId);
    assert.equal(f.states.get('tab-anchor').draft,original.draft);
    if(i%4===0)await measure(i);
  }
  await writeFile(path.join(out,'growth.json'),JSON.stringify({cycles:12,anchorsPreserved:true,checkpoints},null,2));
  assert(checkpoints.at(-1).rendererCount<checkpoints[0].rendererCount+12,'Every closed tab retained an extra renderer');
}finally{
  if(child.exitCode===null)await action('close');
  f.server.closeAllConnections();f.server.close();
}
