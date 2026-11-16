import { spawn, execFile } from 'node:child_process';
import { mkdtemp, mkdir, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import os from 'node:os';
import { promisify } from 'node:util';
import { setTimeout as sleep } from 'node:timers/promises';
import { fixture } from './fixture.mjs';
const exec=promisify(execFile);
const root=fileURLToPath(new URL('../',import.meta.url));
const [exe,directory]=process.argv.slice(2);
if(!exe||!directory)throw Error('Usage: node tools/benchmark.mjs <staged-windows-exe> <results-dir>');
const out=path.resolve(directory);await mkdir(out,{recursive:true});
const home=await mkdtemp(path.join(os.tmpdir(),'desktop-browser-bench-'));
const f=await fixture();
const start=performance.now();
const child=spawn(path.resolve(exe),['--url='+f.url+'/?run=tab-article'],{env:{...process.env,BROWSER_TEST_HOME:home},stdio:'ignore'});
const ps=(script,...argv)=>exec('powershell.exe',['-NoProfile','-ExecutionPolicy','Bypass','-File',path.join(root,'tools',script),'-BrowserPid',String(child.pid),...argv],{windowsHide:true,timeout:90000});
async function waitFor(fn){for(let i=0;i<180;i++){if(fn())return;await sleep(100);}throw Error('Timed out waiting for benchmark fixture');}
try{
  await waitFor(()=>f.states.has('tab-article'));
  const startupReport=Math.round(performance.now()-start);
  await ps('control.ps1','-Action','button','-Value','New tab');
  await ps('control.ps1','-Action','navigate','-Value',f.url+'/second?run=tab-document');
  await waitFor(()=>f.states.has('tab-document'));
  await sleep(6000);
  await ps('sample-windows.ps1','-OutputFile',path.join(out,'memory.json'),'-Samples','10','-IntervalMs','1000');
  const rows=JSON.parse(await readFile(path.join(out,'memory.json'),'utf8'));
  const total=(row,key)=>row.processes.reduce((sum,p)=>sum+Number(p[key]),0);
  const summarize=key=>{const a=rows.map(r=>total(r,key)).sort((a,b)=>a-b);return {medianMiB:(a[4]+a[5])/2/1048576,peakSampledMiB:a.at(-1)/1048576};};
  const meta={date:new Date().toISOString(),compiler:'release',engine:'156.0.30+g6028375',chromium:'156.0.8078.96',os:os.release(),architecture:os.arch(),cpu:os.cpus()[0].model,logicalCPUs:os.cpus().length,ramBytes:os.totalmem(),profile:'new, warm only within this run',tabs:2,layout:'top',blocking:'native, both bundled lists',extensions:[],discard:'no shell-initiated discards',startToFirstFixtureReportMs:startupReport,states:[...f.states.values()],privateWorkingSet:summarize('workingSetPrivateBytes'),privateCommit:summarize('privateCommitBytes'),samplerMs:rows.map(x=>x.samplingMs)};
  await writeFile(path.join(out,'benchmark.json'),JSON.stringify(meta,null,2));
  console.log(JSON.stringify({workingSet:meta.privateWorkingSet,commit:meta.privateCommit}));
}finally{
  if(child.exitCode===null)await ps('control.ps1','-Action','close');
  await waitFor(()=>child.exitCode!==null);f.server.closeAllConnections();f.server.close();
  if(child.exitCode!==0)throw Error('Non-zero browser exit '+child.exitCode);
}
