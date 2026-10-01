// Local test TLS termination. PFX and password are private local environment data.
const fs = require('node:fs');
const https = require('node:https');
const http = require('node:http');
const envFile = process.argv[2];
if (!envFile) throw new Error('Pass the ignored local-test.env path');
const settings = {};
for (const line of fs.readFileSync(envFile, 'utf8').split(/\r?\n/)) {
  if (!line.trim() || line.trimStart().startsWith('#')) continue;
  const match = /^([A-Z_]+)=(.*)$/.exec(line);
  if (!match) throw new Error('Invalid local environment file');
  settings[match[1]] = match[2];
}
if (!/^127\.0\.0\.1:\d+$/.test(settings.SWAN_LISTEN)) throw new Error('Management must use loopback');
const port = Number(settings.SWAN_TEST_TLS_PORT);
if (!Number.isInteger(port) || port < 1024 || port > 65535) throw new Error('Invalid local TLS port');
const upstreamPort = Number(settings.SWAN_LISTEN.split(':')[1]);
const server = https.createServer({
  pfx: fs.readFileSync(settings.SWAN_TEST_TLS_PFX),
  passphrase: settings.SWAN_TEST_TLS_PASSWORD,
  minVersion: 'TLSv1.2',
}, (request, response) => {
  const upstream = http.request({hostname:'127.0.0.1', port:upstreamPort,
    path:request.url, method:request.method, headers:{...request.headers, host:`localhost:${upstreamPort}`}}, result => {
    response.writeHead(result.statusCode, result.headers); result.pipe(response);
  });
  upstream.on('error', () => {if (!response.headersSent) response.writeHead(502);response.end('Local management unavailable');});
  request.on('aborted', () => upstream.destroy());request.pipe(upstream);
});
server.listen(port, '127.0.0.1', () => console.log(`Local HTTPS test listener: https://localhost:${port}`));
