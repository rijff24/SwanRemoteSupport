// Owns only freshly created, loopback-only fixture processes. No production inputs.
const fs=require('node:fs'),path=require('node:path'),https=require('node:https');
const crypto=require('node:crypto'),assert=require('node:assert/strict');
const {spawn,spawnSync}=require('node:child_process');
const root=path.resolve(__dirname,'../..'),settings={};
assert(process.argv[2]&&process.argv[3],'Pass private fixture environment and binary directory');
for(const line of fs.readFileSync(process.argv[2],'utf8').split(/\r?\n/)) {
  if(!line.trim()||line.trimStart().startsWith('#'))continue;
  const match=/^([A-Z_]+)=(.*)$/.exec(line);assert(match,'Invalid environment file');settings[match[1]]=match[2];
}
assert.match(settings.SWAN_LISTEN??'',/^127\.0\.0\.1:\d+$/);
const data=path.resolve(settings.SWAN_DATA_DIR),binary=path.resolve(process.argv[3]);
assert(fs.existsSync(path.join(data,'rotation-fixture.marker')),'Only explicitly marked isolated fixtures are allowed');
assert(!fs.existsSync(path.join(data,'management.sqlite3')),'A fresh, unconfigured fixture is required');
const manager=path.join(binary,'swan-management.exe'),agent=path.join(binary,'swan-agent.exe');
const caFile=path.join(data,'localhost.cer'),ca=new crypto.X509Certificate(fs.readFileSync(caFile)).toString();
const port=Number(settings.SWAN_TEST_TLS_PORT);assert(Number.isInteger(port)&&port>=1024&&port<=65535);
const env={...process.env,...settings,SWAN_TEST_CA_FILE:caFile};
let server,proxy;
function request(endpoint,method='GET',token,body){return new Promise((resolve,reject)=>{const bytes=body===undefined?null:Buffer.from(JSON.stringify(body));const call=https.request(`https://localhost:${port}/api/v1/${endpoint}`,{method,ca,timeout:2000,headers:{'content-type':'application/json',...(token?{authorization:'Bearer '+token}:{}),...(bytes?{'content-length':bytes.length}:{})}},response=>{const chunks=[];response.on('data',c=>chunks.push(c));response.on('end',()=>{try{resolve({status:response.statusCode,body:JSON.parse(Buffer.concat(chunks))});}catch(error){reject(error);}});});call.on('error',reject);call.on('timeout',()=>call.destroy());if(bytes)call.write(bytes);call.end();});}
async function api(endpoint){const result=await request(endpoint);assert.equal(result.status,200);return result.body;}
function totp(secret){const alphabet='ABCDEFGHIJKLMNOPQRSTUVWXYZ234567';let bits=0,value=0,bytes=[];for(const c of secret.replace(/=/g,'')){value=(value<<5)|alphabet.indexOf(c);bits+=5;if(bits>=8){bytes.push((value>>>(bits-8))&255);bits-=8;}}const counter=Buffer.alloc(8);counter.writeBigUInt64BE(BigInt(Math.floor(Date.now()/30000)+1));const hash=crypto.createHmac('sha1',Buffer.from(bytes)).update(counter).digest();const offset=hash[19]&15;return String((hash.readUInt32BE(offset)&0x7fffffff)%1000000).padStart(6,'0');}
function run(executable,args,extra={}){const result=spawnSync(executable,args,{cwd:root,env:{...env,...extra},encoding:'utf8',timeout:60000});assert.equal(result.status,0,`Isolated ${path.basename(executable)} ${args[0]} failed`);return result.stdout;}
function agentRun(directory,args){return run(agent,args,{SWAN_STATE_DIR:directory});}
function readState(directory){return JSON.parse(fs.readFileSync(path.join(directory,'managed-state.json')));}
async function ready(){const deadline=Date.now()+20000;while(Date.now()<deadline){assert.equal(server.exitCode,null,'Owned management process exited');assert.equal(server.signalCode,null,'Owned management process terminated');assert.equal(proxy.exitCode,null,'Owned HTTPS proxy exited');assert.equal(proxy.signalCode,null,'Owned HTTPS proxy terminated');try{return await api('status');}catch{}await new Promise(r=>setTimeout(r,200));}throw new Error('Owned fixture did not become ready');}
async function stop(child){if(!child||child.exitCode!==null||child.signalCode!==null)return;const exited=new Promise(resolve=>child.once('exit',resolve));assert(child.kill(),'Owned fixture process could not be stopped');await exited;}
function start(){server=spawn(manager,[],{cwd:root,env,stdio:'ignore'});}
async function main(){
  start();proxy=spawn(process.execPath,[path.join(__dirname,'local-https.js'),process.argv[2]],{cwd:root,env,stdio:'ignore'});
  const initial=await ready();assert.equal(initial.configured,false);
  run(process.execPath,[path.join(__dirname,'test-company-lifecycle.js'),process.argv[2],binary]);
  const customerDir=path.join(data,'customer-agent'),techDir=path.join(data,'technician-agent'),missedDir=path.join(data,'missed-transition-agent');
  agentRun(missedDir,['setup',path.join(data,'customer-bootstrap.json'),'--accept-company']);
  const original=readState(customerDir),oldKey=original.bootstrap.profile_public_key;
  await stop(server);run(manager,['prepare-profile-key']);start();await ready();
  agentRun(customerDir,['sync']);agentRun(techDir,['sync']);
  const prepared=readState(customerDir),transition=JSON.parse(Buffer.from(prepared.profile.payload,'base64'));
  assert.equal(prepared.bootstrap.profile_public_key,oldKey);assert(transition.next_profile_public_key);assert.notEqual(transition.next_profile_public_key,oldKey);
  assert.equal(readState(techDir).bootstrap.profile_public_key,oldKey);
  // The lifecycle consumed the following MFA step. Wait for a genuinely fresh
  // step instead of replaying that code or weakening the server's MFA checks.
  await new Promise(resolve=>setTimeout(resolve,30000-Date.now()%30000+300));
  const credentials=JSON.parse(fs.readFileSync(path.join(data,'test-admin.private.json')));
  const login=await request('login','POST',null,{username:credentials.username,password:credentials.password,totp_code:totp(credentials.totp_secret)});assert.equal(login.status,200);
  const approval=await request(`devices/${original.device_id}/state`,'PUT',login.body.token,{state:'approved',group:'test-customers'});assert.equal(approval.status,200);
  const proof=crypto.generateKeyPairSync('ed25519').publicKey.export({type:'spki',format:'der'}).subarray(-32).toString('base64');
  const activeGrant=await request('grants','POST',login.body.token,{device_id:original.device_id,proof_public_key:proof,unattended:false});assert.equal(activeGrant.status,200);
  const grantBody=JSON.parse(Buffer.from(activeGrant.body.payload,'base64'));
  assert.equal((await request('grants/claim','POST',original.device_token,{grant:activeGrant.body})).status,200);
  await stop(server);
  const denied=spawnSync(manager,['activate-profile-key'],{cwd:root,env,stdio:'ignore'});assert.notEqual(denied.status,0,'Activation without synchronization confirmation succeeded');
  run(manager,['activate-profile-key','--confirm-clients-synced']);start();const activated=await ready();
  assert.equal(activated.profile_public_key,transition.next_profile_public_key);
  assert(grantBody.expires_at>Math.floor(Date.now()/1000),'Grant expired before the rotation check');
  assert.equal((await request(`grants/${grantBody.grant_id}/renew`,'POST',original.device_token)).status,403,'Rotation renewed an old claimed grant');
  const envelope=await api('profile');const newProfile=JSON.parse(Buffer.from(envelope.payload,'base64'));
  const oldDer=Buffer.concat([Buffer.from('302a300506032b6570032100','hex'),Buffer.from(oldKey,'base64')]);
  assert.equal(crypto.verify(null,Buffer.from(envelope.payload,'base64'),crypto.createPublicKey({key:oldDer,type:'spki',format:'der'}),Buffer.from(envelope.signature,'base64')),false);
  agentRun(customerDir,['sync']);agentRun(techDir,['sync']);
  const after=readState(customerDir);assert.equal(after.bootstrap.profile_public_key,activated.profile_public_key);assert.equal(readState(techDir).bootstrap.profile_public_key,activated.profile_public_key);
  for(const key of ['device_id','device_token','unattended_consent','consent_revision'])assert.equal(after[key],original[key],`Rotation changed ${key}`);
  assert(newProfile.revision>transition.revision);
  const missed=spawnSync(agent,['sync'],{cwd:root,env:{...env,SWAN_STATE_DIR:missedDir},stdio:'ignore',timeout:30000});assert.notEqual(missed.status,0,'A client missing the trusted transition accepted the new key');assert.equal(readState(missedDir).bootstrap.profile_public_key,oldKey);
  await stop(server);start();await ready();agentRun(customerDir,['sync']);assert.equal(readState(customerDir).bootstrap.profile_public_key,activated.profile_public_key);
  const sha256=file=>crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex');
  fs.writeFileSync(path.join(data,'rotation-result.json'),JSON.stringify({passed:true,at:new Date().toISOString(),source_commit:run('git',['rev-parse','HEAD']).trim(),source_dirty:run('git',['status','--porcelain']).trim().length>0,management_sha256:sha256(manager),agent_sha256:sha256(agent),checks:['HTTPS trusted transition','customer and technician key rotation','identity and revoked consent preserved','missed-transition client rejected','activation confirmation required','claimed unexpired grant renewal denied','restart persistence'],native_endpoint_tested:false},null,2));
  console.log('PASS: real HTTPS profile-key rotation with built customer/technician agents. Native apps and live sessions remain unverified.');
}
main().catch(error=>{console.error(error.message);process.exitCode=1;}).finally(async()=>{await stop(server);await stop(proxy);});
