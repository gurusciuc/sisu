import {fixture} from './fixture.mjs';
import {spawn} from 'node:child_process';
import {mkdtemp} from 'node:fs/promises';
import {setTimeout as sleep} from 'node:timers/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import assert from 'node:assert/strict';
const f=await fixture(),home=await mkdtemp(path.join(tmpdir(),'browser-ci-'));
const child=spawn(path.resolve(process.argv[2]),['--url='+f.url+'/?run=posix'],{env:{...process.env,BROWSER_TEST_HOME:home},stdio:'inherit'});
try{
  for(let i=0;i<200&&!f.states.has('posix')&&child.exitCode===null;i++)await sleep(100);
  const r=f.states.get('posix');assert(r,'Missing browser report');assert.equal(r.hidden,'true');
  assert(f.hits.includes('/probe-ad.js?allowed'));assert(!f.hits.includes('/probe-ad.js?direct'));
  console.log(JSON.stringify(r));
}finally{child.kill('SIGTERM');f.server.closeAllConnections();f.server.close();}
