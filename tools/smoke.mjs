import assert from 'node:assert/strict';
import { spawn, execFile } from 'node:child_process';
import { mkdtemp, mkdir, writeFile, readFile } from 'node:fs/promises';
import { tmpdir, release, arch, cpus, totalmem } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { promisify } from 'node:util';
import { setTimeout as sleep } from 'node:timers/promises';
import { fixture } from './fixture.mjs';
const exec = promisify(execFile);
const repo = fileURLToPath(new URL('../', import.meta.url));
const args = Object.fromEntries(process.argv.slice(2).map(x => {const i=x.indexOf('=');return [x.slice(0,i),x.slice(i+1)];}));
const exe=path.resolve(args.exe || path.join(repo,'dist/windows-x64/desktop_browser.exe'));
const home=await mkdtemp(path.join(tmpdir(),'desktop-browser-test-'));
const out=path.resolve(args.out || home);
await mkdir(out,{recursive:true});
const f=await fixture();
const reports=[];
let current;
async function until(pred,label,ms=18000) {
  const start=Date.now();
  while(Date.now()-start<ms){if(pred())return;await sleep(75);}
  throw Error('Timed out: '+label);
}
async function ps(script,...argv) {
  return exec('powershell.exe',['-NoProfile','-ExecutionPolicy','Bypass','-File',path.join(repo,'tools',script),...argv],{windowsHide:true,timeout:35000});
}
function command(action,value) {
  if(process.env.FLOW_TRACE)console.log(action,value??'',current.exitCode);
  return ps('control.ps1','-BrowserPid',String(current.pid),'-Action',action,...(value?['-Value',value]:[]));
}
async function start(run,flags=[]) {
  const when=performance.now(), offset=f.hits.length;
  current=spawn(exe,[...flags,'--url='+f.url+'/?run='+run],{env:{...process.env,BROWSER_TEST_HOME:home},stdio:'ignore'});
  await until(()=>f.states.has(run)||current.exitCode!==null,'page report for '+run);
  assert.equal(current.exitCode,null,'browser exited at startup');
  const state=f.states.get(run);
  const requests=f.hits.slice(offset).filter(s=>!s.startsWith('/report'));
  const entry={run,startToReportMs:Math.round(performance.now()-when),state,requests};
  reports.push(entry);
  assert.equal(state.control,'true');
  assert(requests.includes('/control.js'));
  return entry;
}
async function close() {
  if(!current||current.exitCode!==null)return;
  await command('close');
  await until(()=>current.exitCode!==null,'clean shutdown');
  assert.equal(current.exitCode,0,'clean browser exit');
  current=undefined;
}
async function snapshot(name) {
  await ps('window.ps1','-BrowserPid',String(current.pid),'-Action','capture','-OutputFile',path.join(out,name+'.png'));
}
function native(r){
  assert.equal(r.state.hidden,'true');
  assert(r.requests.includes('/probe-ad.js?allowed'));
  for(const suffix of ['direct','frame','redirect'])assert(!r.requests.includes('/probe-ad.js?'+suffix),suffix+' leaked');
  assert(r.requests.includes('/subframe')&&r.requests.includes('/bounce'));
}
try {
  let r=await start('tab-first');native(r);
  assert.equal(r.state.prior,'empty');
  const doc=r.state.documentId, draft=r.state.draft, width=Number(r.state.width);
  await command('navigate','#settings');
  assert.equal((await command('address')).stdout.trim(),'browser://settings');
  await command('button','Close active tab');
  assert.equal((await command('address')).stdout.trim(),f.url+'/?run=tab-first');
  assert.equal(f.states.get('tab-first').documentId,doc);
  await command('button','New tab');
  await command('navigate',f.url+'/two?run=tab-second');
  await until(()=>f.states.has('tab-second'),'second tab');
  const second=f.states.get('tab-second').documentId;
  await command('button','tab-first');
  await snapshot('tabs-top');
  await command('button','Change tab placement');
  await until(()=>Number(f.states.get('tab-first').width)<width-60,'left tab viewport');
  r=f.states.get('tab-first');
  assert.equal(r.documentId,doc);assert.equal(r.draft,draft);assert.equal(r.hidden,'true');
  await snapshot('tabs-left');
  await command('button','tab-second');
  assert.equal(f.states.get('tab-second').documentId,second);
  await command('button','Change tab placement');
  await until(()=>Number(f.states.get('tab-second').width)===width,'restored viewport');
  await command('navigate',f.url+'/third?run=third');
  await until(()=>f.states.has('third'),'address commit');
  await command('button','Back');
  await sleep(600);
  assert.equal((await command('address')).stdout.trim(),f.url+'/two?run=tab-second');
  await command('button','Forward');
  await sleep(600);
  assert.equal((await command('address')).stdout.trim(),f.url+'/third?run=third');
  const prior=f.states.get('third').documentId;
  await command('button','Reload');
  await until(()=>f.states.get('third').documentId!==prior,'reload changed document');
  await command('button','Pause site blocking');
  let offset=f.hits.length;
  await command('button','Reload');
  await until(()=>f.states.get('third').hidden==='false','site pause');
  assert(f.hits.slice(offset).includes('/probe-ad.js?direct'));
  await command('button','Resume site blocking');
  await command('button','Reload');
  await until(()=>f.states.get('third').hidden==='true','site resume');
  await command('button','Change tab placement');
  reports.push({run:'layout',firstDocument:doc,draft,secondDocument:second,topWidth:width,leftWidth:r.width,restored:true});
  await close();
  r=await start('private-one',['--private']);native(r);assert.equal(r.state.prior,'empty');assert.equal(r.state.cookieBefore,'');await close();
  r=await start('private-two',['--private']);assert.equal(r.state.prior,'empty');assert.equal(r.state.cookieBefore,'');await close();
  r=await start('ordinary-reopened');assert.equal(r.state.prior,'third');assert(r.state.cookieBefore.includes('fixture=third'));
  assert.equal(r.state.width,reports.find(x=>x.run==='layout').leftWidth);
  await close();
  const ext='--extension-lab='+path.join(repo,'fixtures/api-extension');
  r=await start('extension-api',[ext]);
  await until(()=>f.states.get('extension-api').extension,'isolated content script / worker reply');
  const token=f.states.get('extension-api').extension;
  assert.equal(f.states.get('extension-api').isolated,'true');assert.equal(r.state.hidden,'false');
  assert(r.requests.includes('/ads/!rotator/control.js'),'extension lane positive network control');
  const firstTopology=JSON.parse(f.states.get('extension-api').topology);
  await command('button','New tab');
  await command('navigate',f.url+'/second?run=extension-second');
  await until(()=>f.states.get('extension-second')?.topology,'extension-visible second tab');
  const secondTopology=JSON.parse(f.states.get('extension-second').topology);
  assert.notEqual(firstTopology.senderTab,secondTopology.senderTab);
  assert(secondTopology.tabs.some(tab=>tab.id===firstTopology.senderTab));
  reports.push({run:'extension-tab-topology',shellWindows:1,shellTabs:2,firstTopology,secondTopology,
    matchesShellWindow:firstTopology.senderWindow===secondTopology.senderWindow});
  await close();await start('extension-restart',[ext]);
  await until(()=>f.states.get('extension-restart').extension===token,'worker storage after restart');await close();
  if(args.ubol){
    r=await start('ubol',['--extension-lab='+path.resolve(args.ubol)]);
    assert.equal(r.state.hidden,'false');assert(r.requests.includes('/probe-ad.js?direct'));
    assert(!r.requests.includes('/ads/!rotator/control.js'),'uBOL DNR target reached server');await close();
  }
  console.log('Browser workflow assertions passed.');
} catch(error){reports.push({failure:String(error),exit:current?.exitCode});process.exitCode=1;console.error(error);}
finally {
  try {await close();} catch(e){console.error(e);process.exitCode=1;}
  f.server.closeAllConnections();f.server.close();
  await writeFile(path.join(out,'flows.json'),JSON.stringify({engine:'CEF 156.0.30 / Chromium 156.0.8078.96',os:release(),arch:arch(),reports},null,2));
}
