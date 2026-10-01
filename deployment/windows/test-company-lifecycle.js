// Real local HTTPS/API and built-agent integration. Requires a fresh test server.
// This does not substitute for native remote desktop or clean-machine tests.
const fs=require('node:fs'), path=require('node:path'), https=require('node:https');
const crypto=require('node:crypto'), assert=require('node:assert/strict');
const {spawnSync}=require('node:child_process');
const root=path.resolve(__dirname,'../..');
const envFile=process.argv[2];
if(!envFile)throw new Error('Pass a private local-test.env file for a fresh isolated server');
const settings={};
for(const line of fs.readFileSync(envFile,'utf8').split(/\r?\n/)){
  if(!line.trim()||line.trimStart().startsWith('#'))continue;
  const match=/^([A-Z_]+)=(.*)$/.exec(line);if(!match)throw new Error('Invalid test environment data');
  settings[match[1]]=match[2];
}
assert.match(settings.SWAN_LISTEN??'',/^127\.0\.0\.1:\d+$/);
assert(settings.SWAN_DATA_DIR,'A separate test data directory is required');
const data=path.resolve(settings.SWAN_DATA_DIR);
const agent=path.join(root,'product/target/debug/swan-agent.exe');
const ca=new crypto.X509Certificate(fs.readFileSync(path.join(data,'localhost.cer'))).toString();
const port=Number(settings.SWAN_TEST_TLS_PORT);assert(Number.isInteger(port)&&port>=1024&&port<=65535);
const base=`https://localhost:${port}`;
const alphabet='ABCDEFGHIJKLMNOPQRSTUVWXYZ234567';
function base32(bytes){let bits=0,value=0,result='';for(const byte of bytes){value=(value<<8)|byte;bits+=8;while(bits>=5){result+=alphabet[(value>>>(bits-5))&31];bits-=5;}}if(bits)result+=alphabet[(value<<(5-bits))&31];return result;}
function decode32(text){let bits=0,value=0,result=[];for(const c of text.replace(/=/g,'')){value=(value<<5)|alphabet.indexOf(c);bits+=5;if(bits>=8){result.push((value>>>(bits-8))&255);bits-=8;}}return Buffer.from(result);}
function totp(secret,offset=0){const step=BigInt(Math.floor(Date.now()/30000)+offset);const counter=Buffer.alloc(8);counter.writeBigUInt64BE(step);const hash=crypto.createHmac('sha1',decode32(secret)).update(counter).digest();const pos=hash[19]&15;return String((hash.readUInt32BE(pos)&0x7fffffff)%1000000).padStart(6,'0');}
function api(endpoint,method='GET',token,body){return new Promise((resolve,reject)=>{
  const bytes=body===undefined?null:Buffer.from(JSON.stringify(body));
  const request=https.request(base+'/api/v1/'+endpoint,{method,ca,headers:{'content-type':'application/json',...(token?{authorization:'Bearer '+token}:{}),...(bytes?{'content-length':bytes.length}:{})}},response=>{
    const chunks=[];response.on('data',chunk=>chunks.push(chunk));response.on('end',()=>{try{resolve({status:response.statusCode,body:JSON.parse(Buffer.concat(chunks))});}catch(error){reject(error);}});
  });request.on('error',reject);if(bytes)request.write(bytes);request.end();
});}
function runAgent(directory,args,extra={}){const result=spawnSync(agent,args,{env:{...process.env,SWAN_STATE_DIR:directory,SWAN_TEST_CA_FILE:path.join(data,'localhost.cer'),...extra},encoding:'utf8',timeout:30000});assert.equal(result.status,0,`Agent ${args[0]} failed: ${result.stderr}`);return result.stdout;}
function state(directory){return JSON.parse(fs.readFileSync(path.join(directory,'managed-state.json')));}
async function main(){
  const status=(await api('status')).body;assert.equal(status.configured,false,'Refuse to change an already configured test company');
  const password=crypto.randomBytes(24).toString('hex'), secret=base32(crypto.randomBytes(20));
  const brand={display_name:'Local Test Company Support',primary_color:'#007F82',logo_svg:'',support_url:base,consent_text:'Local test support requires approval.'};
  const transport=crypto.generateKeyPairSync('ed25519').publicKey.export({type:'spki',format:'der'}).subarray(-32).toString('base64');
  const profile={schema:1,company_id:'setup',revision:1,issued_at:Math.floor(Date.now()/1000),expires_at:Math.floor(Date.now()/1000)+3600,management_url:base,rendezvous:'localhost:22116',relay:'localhost:22117',transport_public_key:transport,customer:brand,technician:{...brand,display_name:'Local Test Technician'},allow_unattended:true,updates_paused:true,rollout_percent:100,maintenance_start_utc:0,maintenance_end_utc:0,update_channel:'test',next_profile_public_key:null};
  const setup=await api('setup','POST',fs.readFileSync(path.join(data,'setup-token.txt'),'utf8').trim(),{profile,username:'test-admin',password,totp_secret:secret,totp_code:totp(secret)});assert.equal(setup.status,200);
  fs.writeFileSync(path.join(data,'test-admin.private.json'),JSON.stringify({username:'test-admin',password,totp_secret:secret}));
  const login=await api('login','POST',null,{username:'test-admin',password,totp_code:totp(secret,1)});assert.equal(login.status,200);const admin=login.body.token;
  const signed=(await api('profile')).body;const current=JSON.parse(Buffer.from(signed.payload,'base64'));
  const bootstrap={schema:1,edition:'customer',company_id:current.company_id,management_url:base,profile_public_key:status.profile_public_key,release_public_key:''};
  const customerDir=path.join(data,'customer-agent'), techDir=path.join(data,'technician-agent');
  const customerBootstrap=path.join(data,'customer-bootstrap.json');fs.writeFileSync(customerBootstrap,JSON.stringify(bootstrap));
  runAgent(customerDir,['setup',customerBootstrap,'--accept-company']);runAgent(customerDir,['enroll','Local HTTPS test device','987654321']);
  const customer=state(customerDir);assert.equal(customer.unattended_consent,false);
  const proof=crypto.generateKeyPairSync('ed25519').publicKey.export({type:'spki',format:'der'}).subarray(-32).toString('base64');
  const ticket={device_id:customer.device_id,proof_public_key:proof,unattended:false};
  assert.equal((await api('grants','POST',admin,ticket)).status,403,'Pending device was accessible');
  assert.equal((await api(`devices/${customer.device_id}/state`,'PUT',admin,{state:'approved',group:'test-customers'})).status,200);
  const technicianPassword=crypto.randomBytes(24).toString('hex');
  const technician=(await api('users','POST',admin,{username:'test-technician',password:technicianPassword,role:'technician'})).body;
  const techBootstrap=path.join(data,'technician-bootstrap.json');fs.writeFileSync(techBootstrap,JSON.stringify({...bootstrap,edition:'technician'}));runAgent(techDir,['setup',techBootstrap,'--accept-company']);
  const technicianLogin=JSON.parse(runAgent(techDir,['login','test-technician'],{SWAN_LOGIN_PASSWORD:technicianPassword,SWAN_LOGIN_TOTP:totp(technician.totp_secret)}));
  const auth={SWAN_TECHNICIAN_TOKEN:technicianLogin.token};assert.equal(JSON.parse(runAgent(techDir,['devices'],auth)).length,0,'Technician without group access saw device');
  assert.equal((await api(`groups/test-customers/users/${technician.id}`,'PUT',admin,{})).status,200);
  assert.equal(JSON.parse(runAgent(techDir,['devices'],auth)).length,1);
  assert.equal((await api('grants','POST',technicianLogin.token,{...ticket,unattended:true})).status,403,'Unattended grant issued without consent');
  const grant=await api('grants','POST',technicianLogin.token,ticket);assert.equal(grant.status,200);
  assert.equal((await api('grants/claim','POST',customer.device_token,{grant:grant.body})).status,200);
  assert.equal((await api('grants/claim','POST',customer.device_token,{grant:grant.body})).status,403,'Grant replay accepted');
  const history=JSON.parse(runAgent(techDir,['history'],auth));assert.equal(history.length,1);assert.equal(history[0].claimed,true);assert.equal(history[0].device_id,customer.device_id);
  current.customer.display_name='Changed Local Test Branding';assert.equal((await api('profile','PUT',admin,current)).status,200);
  runAgent(customerDir,['sync']);const updated=JSON.parse(Buffer.from(state(customerDir).profile.payload,'base64'));assert.equal(updated.customer.display_name,'Changed Local Test Branding');assert.equal(updated.revision,2);
  assert.equal((await api(`devices/${customer.device_id}/state`,'PUT',admin,{state:'revoked',group:'test-customers'})).status,200);
  assert.equal((await api('grants','POST',technicianLogin.token,ticket)).status,403,'Revoked device accessible');
  runAgent(techDir,['logout'],auth);assert.equal((await api('devices','GET',technicianLogin.token)).status,401,'Logged-out credential still authorized');
  const source=spawnSync('git',['rev-parse','HEAD'],{cwd:root,encoding:'utf8'});
  const dirty=spawnSync('git',['status','--porcelain'],{cwd:root,encoding:'utf8'});
  fs.writeFileSync(path.join(data,'lifecycle-result.json'),JSON.stringify({passed:true,at:new Date().toISOString(),source_commit:source.status===0?source.stdout.trim():'unknown',source_dirty:dirty.status===0?dirty.stdout.trim().length>0:null,agent_sha256:crypto.createHash('sha256').update(fs.readFileSync(agent)).digest('hex'),management_sha256:crypto.createHash('sha256').update(fs.readFileSync(path.join(root,'product/target/debug/swan-management.exe'))).digest('hex'),checks:['HTTPS certificate validation','fresh company setup','built agent bootstrap and enrollment','pending approval denial','technician MFA login and group inventory','technician session history','logout revokes credentials','unattended consent denial','grant replay denial','signed branding sync','device revocation']},null,2));
  console.log('PASS: real local HTTPS company lifecycle and built-agent checks. Native sessions and installers remain separate tests.');
}
main().catch(error=>{console.error(error.message);process.exitCode=1;});
