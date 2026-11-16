import {fixture} from './fixture.mjs';
import {spawn,execFile} from 'node:child_process';
import {writeFile,mkdtemp} from 'node:fs/promises';
import {setTimeout as sleep} from 'node:timers/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {promisify} from 'node:util';
const f=await fixture(),home=await mkdtemp(path.join(tmpdir(),'browser-capabilities-'));
const p=spawn(path.resolve(process.argv[2]),['--url='+f.url+'/capabilities'],{env:{...process.env,BROWSER_TEST_HOME:home},stdio:'ignore'});
try{
  for(let i=0;i<370&&!f.states.has('capabilities');i++)await sleep(100);
  if(!f.states.has('capabilities'))throw Error('Missing capability report');
  const checks=JSON.parse(f.states.get('capabilities').checks);
  const requests=f.hits.filter(x=>!x.startsWith('/report'));
  const result={checks,requests,workerNetworkProtected: ['dedicated','service'].every(k=>requests.includes('/control.js?'+k)&&!requests.includes('/ads/!rotator/'+k))};
  console.log(result);await writeFile(process.argv[3],JSON.stringify(result,null,2));
}finally{
  if(p.exitCode===null)await promisify(execFile)('powershell.exe',['-NoProfile','-ExecutionPolicy','Bypass','-File',fileURLToPath(new URL('./control.ps1',import.meta.url)),'-BrowserPid',String(p.pid),'-Action','close']);
  f.server.closeAllConnections();f.server.close();
}
