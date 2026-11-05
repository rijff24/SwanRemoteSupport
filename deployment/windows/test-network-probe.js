'use strict';
// Exercise the compiled CLI and real TLS/signatures. TCP/UDP peers below are
// scripted diagnostics, not native-session or public-connectivity qualification.
const fs = require('node:fs'), path = require('node:path');
const https = require('node:https'), net = require('node:net'), dgram = require('node:dgram');
const crypto = require('node:crypto'), assert = require('node:assert/strict');
const {spawn} = require('node:child_process');
const [environment, executable, resultPath] = process.argv.slice(2);
assert(environment && executable && resultPath, 'Pass private TLS environment, management executable and new result path');
assert(!fs.existsSync(resultPath), 'Preserve existing evidence');
const settings = Object.fromEntries(fs.readFileSync(environment, 'utf8').split(/\r?\n/).filter(Boolean).map(line => {
  const match = /^([A-Z_]+)=(.*)$/.exec(line); assert(match, 'Invalid private test environment'); return [match[1], match[2]];
}));
const directory = path.dirname(path.resolve(resultPath)), sentinel = path.join(directory, 'must-not-create-company-state');
assert(!fs.existsSync(sentinel), 'State sentinel already exists');
const key = crypto.generateKeyPairSync('ed25519');
const publicKey = key.publicKey.export({type:'spki', format:'der'}).subarray(-32).toString('base64');
let mode = 'valid', profile, child, udp, udpOpen = false;
let transportConnections = 0, profileRequests = 0;
const listeners = [], checks = [], credentials = [], requestedPaths = [];
const listen = (server, port = 0) => new Promise((resolve, reject) => {
  server.once('error', reject); server.listen(port, '127.0.0.1', () => { server.removeListener('error', reject); resolve(server.address().port); });
});
const tcp = () => net.createServer(socket => { ++transportConnections; socket.end(); });
async function adjacentTcpPorts() {
  for (let attempt=0; attempt<10; ++attempt) {
    const nat=tcp(), id=tcp(); const port=await listen(nat);
    try {
      if (port===65535) continue;
      await listen(id,port+1); listeners.push(nat,id); return port;
    } catch (error) { if (error.code!=='EADDRINUSE') throw error; }
    finally { if (!id.listening) await new Promise(resolve => nat.close(resolve)); }
  }
  throw new Error('No adjacent loopback TCP fixture ports available');
}
const server = https.createServer({pfx:fs.readFileSync(settings.SWAN_TEST_TLS_PFX), passphrase:settings.SWAN_TEST_TLS_PASSWORD}, (request, response) => {
  requestedPaths.push(request.url);
  credentials.push(!!request.headers.authorization || !!request.headers.cookie);
  if (request.url !== '/api/v1/profile') { response.writeHead(404); response.end(); return; }
  ++profileRequests;
  if (mode === 'redirect') { response.writeHead(302, {location:'/redirected'}); response.end(); return; }
  const value = {...profile};
  if (mode === 'expired') { value.issued_at -= 7200; value.expires_at = value.issued_at + 3600; }
  if (mode === 'future') { value.issued_at += 3600; value.expires_at += 3600; }
  if (mode === 'zero-revision') value.revision = 0;
  if (mode === 'wrong-company') value.company_id = 'another-company';
  let payload = Buffer.from(JSON.stringify(value));
  const signature = crypto.sign(null, payload, key.privateKey).toString('base64');
  if (mode === 'tampered') payload = Buffer.from(JSON.stringify({...value, revision:999}));
  response.setHeader('content-type','application/json');
  response.end(JSON.stringify({payload:payload.toString('base64'), signature}));
});
async function command(label, args, expected, expectedReport) {
  const before = transportConnections;
  child = spawn(path.resolve(executable), args, {windowsHide:true, stdio:['ignore','pipe','pipe'], env:{...process.env, SWAN_DATA_DIR:sentinel, SWAN_TECHNICIAN_TOKEN:'not-a-real-token-do-not-send'}});
  let output = '', error = '';
  child.stdout.on('data', data => { output += data; }); child.stderr.on('data', data => { error += data; });
  const timer = setTimeout(() => child.kill(), 30000);
  let code;
  try { code = await new Promise((resolve, reject) => {child.once('error', reject); child.once('exit', resolve);}); }
  finally {clearTimeout(timer);}
  assert.equal(code, expected, `${label}: unexpected CLI exit (${error.slice(0,300)})`);
  assert(!fs.existsSync(sentinel), `${label}: checker opened company state`);
  if (expectedReport !== undefined) {
    const report = JSON.parse(output); assert.equal(report.requested_checks_passed, expectedReport);
    assert.equal(report.outside_vantage_independently_attested, false);
    assert.equal(report.native_direct_and_relay_sessions_verified, false);
    assert.equal(report.connected_addresses_public_candidates, false);
    if (label !== 'malformed-udp') assert.equal(report.all_endpoint_checks_passed, true);
    else assert.equal(report.udp_check.reachable, false);
  } else if (label !== 'help') {
    assert.equal(transportConnections, before, `${label}: untrusted profile reached transport`);
    assert.equal(output.trim(), '', `${label}: emitted a success report for invalid trust`);
  }
  checks.push(label);
}
async function main() {
  const natPort = await adjacentTcpPorts();
  const relay = tcp(); listeners.push(relay); const relayPort = await listen(relay);
  udp = dgram.createSocket('udp4');
  udp.on('message', (bytes, peer) => {
    assert.deepEqual(bytes, Buffer.from([0x7a,0x06,0x12,0x01,0x00,0x1a,0x01,0x00]));
    udp.send(mode === 'malformed-udp' ? Buffer.from([0]) : Buffer.from([0x82,0x01,0x02,0x08,0x02]), peer.port, peer.address);
  });
  await new Promise((resolve,reject) => {udp.once('error',reject); udp.bind(natPort+1,'127.0.0.1',resolve);}); udpOpen = true;
  const httpsPort = await listen(server); listeners.push(server);
  const brand = {display_name:'Probe fixture',primary_color:'#007777',logo_svg:'<svg/>',support_url:'',consent_text:'Approve support.',support_contacts:'',shortcuts:[]};
  const now = Math.floor(Date.now()/1000), url = `https://localhost:${httpsPort}`;
  profile = {schema:1,company_id:'probe-fixture',revision:1,issued_at:now,expires_at:now+3600,management_url:url,rendezvous:`127.0.0.1:${natPort+1}`,relay:`127.0.0.1:${relayPort}`,transport_public_key:publicKey,customer:brand,technician:brand,allow_unattended:false,updates_paused:true,rollout_percent:0,maintenance_start_utc:0,maintenance_end_utc:0,update_channel:'test',next_profile_public_key:null};
  const bootstrap = {schema:1,edition:'customer',company_id:profile.company_id,management_url:url,profile_public_key:publicKey,release_public_key:''};
  const bootstrapPath = path.join(directory,'probe-bootstrap.json'); fs.writeFileSync(bootstrapPath, JSON.stringify(bootstrap), {flag:'wx'});
  const args = ['check-network',bootstrapPath,'--ca-file',settings.SWAN_TEST_CA_FILE];
  const pemPath = path.join(directory,'root.pem');
  fs.writeFileSync(pemPath,new crypto.X509Certificate(fs.readFileSync(settings.SWAN_TEST_CA_FILE)).toString(),{flag:'wx'});
  await command('help',['check-network','--help'],0);
  await command('trusted-private',args,0,true);
  await command('trusted-pem',['check-network',bootstrapPath,'--ca-file',pemPath],0,true);
  await command('private-cannot-qualify-outside',[...args,'--outside-network'],1,false);
  for (const label of ['expired','future','zero-revision','wrong-company','tampered','redirect']) {mode=label; await command(label,args,1);}
  mode='valid';
  const wrongKeyPath = path.join(directory,'wrong-key-bootstrap.json');
  const wrongKey = crypto.generateKeyPairSync('ed25519').publicKey.export({type:'spki',format:'der'}).subarray(-32).toString('base64');
  fs.writeFileSync(wrongKeyPath,JSON.stringify({...bootstrap,profile_public_key:wrongKey}),{flag:'wx'});
  await command('wrong-pinned-key',['check-network',wrongKeyPath,'--ca-file',settings.SWAN_TEST_CA_FILE],1);
  const wrongHostPath = path.join(directory,'wrong-host-bootstrap.json');
  fs.writeFileSync(wrongHostPath,JSON.stringify({...bootstrap,management_url:`https://127.0.0.1:${httpsPort}`}),{flag:'wx'});
  await command('certificate-hostname-mismatch',['check-network',wrongHostPath,'--ca-file',settings.SWAN_TEST_CA_FILE],1);
  mode='valid'; await command('untrusted-ca',['check-network',bootstrapPath],1);
  await command('insecure-option-rejected',[...args,'--insecure'],1);
  mode='malformed-udp'; await command(mode,args,1,false);
  assert(profileRequests >= 10); assert(credentials.every(value => !value), 'Checker transmitted account credentials');
  assert(!requestedPaths.includes('/redirected'), 'Checker followed a profile redirect');
  fs.writeFileSync(resultPath, JSON.stringify({passed:true,checks,management_sha256:crypto.createHash('sha256').update(fs.readFileSync(executable)).digest('hex'),no_company_state_created:true,no_request_credentials:true,real_https_and_ed25519:true,transport_peers:'scripted fixture; not native sessions',native_or_external_connectivity_qualified:false},null,2), {flag:'wx'});
  console.log('PASS: compiled checker TLS, signed profiles, trust rejection, private-scope enforcement and no company state or credentials.');
}
main().catch(error => {console.error(error.message); process.exitCode=1;}).finally(async () => {
  if (child && child.exitCode === null && child.signalCode === null) child.kill();
  server.closeAllConnections();
  for (const listener of listeners) if (listener.listening) await new Promise(resolve => listener.close(resolve));
  if (udpOpen) udp.close();
});
