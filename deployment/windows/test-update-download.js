'use strict';
// Real HTTPS transport regression; no installer or helper code is executed.
const fs = require('node:fs'), path = require('node:path');
const https = require('node:https'), crypto = require('node:crypto');
const assert = require('node:assert/strict');
const {spawn, spawnSync} = require('node:child_process');
const [environment, executable, resultPath] = process.argv.slice(2);
assert(environment && executable && resultPath, 'Pass private TLS environment, Rust agent test executable, and new result path');
assert(!fs.existsSync(resultPath), 'Do not overwrite previous evidence');
const settings = {};
for (const line of fs.readFileSync(environment, 'utf8').split(/\r?\n/)) {
  if (!line.trim() || line.trimStart().startsWith('#')) continue;
  const match = /^([A-Z_]+)=(.*)$/.exec(line);
  assert(match, 'Invalid environment file'); settings[match[1]] = match[2];
}
assert(settings.SWAN_TEST_TLS_PFX && settings.SWAN_TEST_CA_FILE, 'Private test TLS paths required');
const payload = Buffer.from('Swan HTTPS recovery download fixture\n');
const requests = [];
const server = https.createServer({pfx: fs.readFileSync(settings.SWAN_TEST_TLS_PFX), passphrase: settings.SWAN_TEST_TLS_PASSWORD}, (request, response) => {
  requests.push({path: request.url, authorization: !!request.headers.authorization, cookie: !!request.headers.cookie});
  if (request.url === '/artifact') { response.writeHead(302, {location: '/payload'}); response.end(); }
  else if (request.url === '/payload') { response.end(payload); }
  else if (request.url === '/tampered') { response.end('tampered download'); }
  else if (request.url === '/partial') {
    response.writeHead(200, {'content-length': payload.length});
    response.flushHeaders(); response.write(payload.subarray(0, 8));
    setImmediate(() => response.destroy());
  } else { response.writeHead(404); response.end(); }
});
let child;
async function main() {
  await new Promise((resolve, reject) => { server.once('error', reject); server.listen(0, '127.0.0.1', resolve); });
  const base = `https://localhost:${server.address().port}`;
  child = spawn(path.resolve(executable), ['update::tests::https_download_recovery_preserves_staging_on_tamper_and_interruption', '--exact', '--ignored', '--nocapture'], {
    env: {...process.env, SWAN_TEST_CA_FILE: settings.SWAN_TEST_CA_FILE, SWAN_TEST_DOWNLOAD_BASE: base, SWAN_TECHNICIAN_TOKEN: 'not-a-real-credential-downloads-must-not-send-it'},
    stdio: ['ignore', 'pipe', 'pipe'], windowsHide: true,
  });
  let output = '';
  child.stdout.on('data', data => { output += data; });
  child.stderr.on('data', data => { output += data; });
  const code = await new Promise((resolve, reject) => { child.once('error', reject); child.once('exit', resolve); });
  assert.equal(code, 0, 'HTTPS download test failed');
  assert.match(output, /1 passed; 0 failed/);
  for (const route of ['/artifact', '/payload', '/tampered', '/partial']) assert(requests.some(request => request.path === route), `Missing ${route} request`);
  assert(requests.every(request => !request.authorization && !request.cookie), 'Artifact requests carried credentials');
  const hash = file => crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex');
  const root = path.resolve(__dirname, '../..');
  const commit = spawnSync('git', ['rev-parse', 'HEAD'], {cwd: root, encoding: 'utf8'});
  const dirty = spawnSync('git', ['status', '--porcelain'], {cwd: root, encoding: 'utf8'});
  assert.equal(commit.status, 0); assert.equal(dirty.status, 0);
  fs.writeFileSync(resultPath, JSON.stringify({passed: true, at: new Date().toISOString(), source_commit: commit.stdout.trim(), source_dirty: dirty.stdout.trim().length > 0, agent_test_sha256: hash(executable), tls_identity_sha256: hash(settings.SWAN_TEST_CA_FILE), checks: ['real HTTPS redirect', 'exact artifact hash', 'tamper rejection retains staging', 'interrupted response retains staging', 'no artifact credentials', 'temporary staging cleaned'], requests, native_installer_executed: false}, null, 2), {flag: 'wx'});
  console.log('PASS: HTTPS redirect, tamper/interruption rejection, complete staging retained, no artifact credentials.');
}
main().catch(error => { console.error(error.message); process.exitCode = 1; }).finally(async () => {
  if (child && child.exitCode === null && child.signalCode === null) {
    const exited = new Promise(resolve => child.once('exit', resolve)); child.kill(); await exited;
  }
  server.closeAllConnections();
  if (server.listening) await new Promise(resolve => server.close(resolve));
});
