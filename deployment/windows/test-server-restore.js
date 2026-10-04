// Real HTTPS restore rehearsal for an isolated lifecycle fixture, not production.
const fs=require('node:fs'),path=require('node:path'),https=require('node:https');
const crypto=require('node:crypto'),assert=require('node:assert/strict');
function settings(file){const values={};for(const line of fs.readFileSync(file,'utf8').split(/\r?\n/)){if(!line.trim()||line.trimStart().startsWith('#'))continue;const match=/^([A-Z_]+)=(.*)$/.exec(line);assert(match,'Invalid private test environment');values[match[1]]=match[2];}assert.match(values.SWAN_LISTEN,/^127\.0\.0\.1:\d+$/);return values;}
function endpoint(values){const data=path.resolve(values.SWAN_DATA_DIR),port=Number(values.SWAN_TEST_TLS_PORT);assert(Number.isInteger(port)&&port>=1024&&port<=65535);return {data,base:`https://localhost:${port}`,ca:new crypto.X509Certificate(fs.readFileSync(path.join(data,'localhost.cer'))).toString()};}
function api(server,route,method='GET',token,body){return new Promise((resolve,reject)=>{const bytes=body===undefined?null:Buffer.from(JSON.stringify(body));const request=https.request(server.base+'/api/v1/'+route,{method,ca:server.ca,headers:{'content-type':'application/json',...(token?{authorization:'Bearer '+token}:{}),...(bytes?{'content-length':bytes.length}:{})}},response=>{const chunks=[];response.on('data',chunk=>chunks.push(chunk));response.on('end',()=>{try{resolve({status:response.statusCode,body:JSON.parse(Buffer.concat(chunks))});}catch(error){reject(error);}});});request.setTimeout(15000,()=>request.destroy(new Error('Test request timed out')));request.on('error',reject);if(bytes)request.write(bytes);request.end();});}
function totp(secret){let bits=0,value=0,decoded=[];for(const c of secret.replace(/=/g,'')){const index='ABCDEFGHIJKLMNOPQRSTUVWXYZ234567'.indexOf(c);assert(index>=0);value=(value<<5)|index;bits+=5;if(bits>=8){decoded.push((value>>>(bits-8))&255);bits-=8;}}const counter=Buffer.alloc(8);counter.writeBigUInt64BE(BigInt(Math.floor(Date.now()/30000)+1));const hash=crypto.createHmac('sha1',Buffer.from(decoded)).update(counter).digest(),offset=hash[19]&15;return String((hash.readUInt32BE(offset)&0x7fffffff)%1000000).padStart(6,'0');}
function verifiedProfile(envelope,key){const der=Buffer.concat([Buffer.from('302a300506032b6570032100','hex'),Buffer.from(key,'base64')]);assert(crypto.verify(null,Buffer.from(envelope.payload,'base64'),{key:der,type:'spki',format:'der'},Buffer.from(envelope.signature,'base64')),'Restored profile signature is invalid');return JSON.parse(Buffer.from(envelope.payload,'base64'));}
async function main(){
  assert(process.argv[2]&&process.argv[3],'Pass original and restored private test environment files');
  const original=endpoint(settings(process.argv[2])),restored=endpoint(settings(process.argv[3]));assert.notEqual(original.data,restored.data);assert.notEqual(original.base,restored.base);
  const status=(await api(original,'status')).body,newStatus=(await api(restored,'status')).body;
  assert.equal(status.configured,true);assert.deepEqual(newStatus,status,'Company setup or trust key changed on restore');
  const profile=verifiedProfile((await api(original,'profile')).body,status.profile_public_key);
  assert.deepEqual(verifiedProfile((await api(restored,'profile')).body,status.profile_public_key),profile,'Signed configuration changed on restore');
  const identity=JSON.parse(fs.readFileSync(path.join(original.data,'test-admin.private.json')));
  const loginBody={username:identity.username,password:identity.password,totp_code:totp(identity.totp_secret)};
  const oldLogin=await api(original,'login','POST',null,loginBody),newLogin=await api(restored,'login','POST',null,loginBody);
  assert.equal(oldLogin.status,200,'Original test MFA login failed');assert.equal(newLogin.status,200,'Restored MFA login failed');
  const oldToken=oldLogin.body.token,newToken=newLogin.body.token;
  try {
    for(const route of ['devices','users','groups/test-customers/permissions']){
      const before=await api(original,route,'GET',oldToken),after=await api(restored,route,'GET',newToken);
      assert.equal(before.status,200);assert.equal(after.status,200);assert.deepEqual(after.body,before.body,`Restored ${route} differs`);
    }
    const device=JSON.parse(fs.readFileSync(path.join(original.data,'customer-agent/managed-state.json')));
    assert.equal((await api(restored,'device/status','GET',device.device_token)).status,401,'Restore reenabled a revoked device');
    const proof=crypto.generateKeyPairSync('ed25519').publicKey.export({type:'spki',format:'der'}).subarray(-32).toString('base64');
    assert.equal((await api(restored,'grants','POST',newToken,{device_id:device.device_id,proof_public_key:proof,unattended:false})).status,403,'Restored server issued a grant for a revoked device');
    fs.writeFileSync(path.join(restored.data,'restore-result.json'),JSON.stringify({passed:true,at:new Date().toISOString(),company_id:profile.company_id,checks:['HTTPS certificate validation','configuration signing key preserved','signed profile preserved','administrator MFA login','accounts and devices preserved','group capability policy preserved','revoked device denied','revoked-device grant denied']},null,2));
  } finally {await api(original,'logout','POST',oldToken);await api(restored,'logout','POST',newToken);}
  console.log('PASS: real HTTPS server restore preserves company trust, MFA, device state and group policy; revoked access remains denied.');
}
main().catch(error=>{console.error(error.message);process.exitCode=1;});
