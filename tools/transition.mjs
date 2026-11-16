import { fixture } from './fixture.mjs';
import { spawn, execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { setTimeout as sleep } from 'node:timers/promises';
import { mkdtemp } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const [exe,out]=process.argv.slice(2);
if(!exe||!out)throw Error('Expected a staged browser and an output JSON path');
const f=await fixture(), home=await mkdtemp(path.join(os.tmpdir(),'desktop-transition-'));
const child=spawn(path.resolve(exe),['--url='+f.url+'/?run=transition'],{env:{...process.env,BROWSER_TEST_HOME:home},stdio:'ignore'});
const tools=fileURLToPath(new URL('./',import.meta.url));
const ps=(script,...args)=>promisify(execFile)('powershell.exe',['-NoProfile','-ExecutionPolicy','Bypass','-File',path.join(tools,script),'-BrowserPid',String(child.pid),...args],{windowsHide:true,timeout:30000});
try{
  for(let i=0;i<150&&!f.states.has('transition');i++)await sleep(100);
  if(!f.states.has('transition'))throw Error('Fixture failed to render');
  await ps('transition.ps1','-OutputFile',path.resolve(out));
}finally{
  if(child.exitCode===null)await ps('control.ps1','-Action','close');
  f.server.closeAllConnections();f.server.close();
}
