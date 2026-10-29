// Browser contract checks; real reporting and native update tests remain separate.
'use strict';
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const vm=require('node:vm');

async function main(){
  const nodes=new Map(),timers=new Map(),requests=[];
  let timer=0,hold=false,release,releaseLogout;
  class Element{
    addEventListener(){}
    set innerHTML(value){this.html=value;for(const match of value.matchAll(/\bid="([^"]+)"/g))nodes.set('#'+match[1],new Element());}
    get innerHTML(){return this.html;}
  }
  for(const id of ['content','notice','title','description','nav','logout'])nodes.set('#'+id,new Element());
  const rows=[
    {owner_name:'<script>customer</script>',owner_state:'approved',edition:'customer',received_at:0,report:{installed_version:'1.5.7',phase:'downloading',downloaded_bytes:50,total_bytes:100}},
    {owner_name:'Company technician',owner_state:'active',edition:'technician',received_at:Date.now()/1000,report:{installed_version:'1.5.8',phase:'installing',downloaded_bytes:0,total_bytes:null}},
  ];
  const response=value=>({ok:true,json:async()=>value});
  const context=vm.createContext({
    document:{querySelector:selector=>nodes.get(selector)||null,querySelectorAll:()=>[]},
    setTimeout:(callback,delay)=>{assert.equal(delay,10000);timers.set(++timer,callback);return timer;},
    clearTimeout:id=>timers.delete(id),
    fetch:async(url,options)=>{
      requests.push({url,options});
      assert.equal(options.headers.Authorization,'Bearer fixture-session');
      if(url==='/api/v1/logout')return new Promise(resolve=>{releaseLogout=()=>resolve(response({ok:true}));});
      assert.match(url,/^\/api\/v1\/installations\?offset=(0|200)$/);
      if(hold)return new Promise(resolve=>{release=()=>resolve(response({reports:rows,next_offset:200}));});
      return response(url.endsWith('=0')?{reports:rows,next_offset:200}:{reports:[],next_offset:null});
    },
  });
  const source=fs.readFileSync(path.join(__dirname,'../../product/management/web/app.js'),'utf8').replace(/\r\n/g,'\n');
  assert.ok(source.endsWith('initialize();\n'));
  vm.runInContext(source.replace(/initialize\(\);\n$/,''),context);
  await vm.runInContext("accessToken='fixture-session';view('installations')",context);
  let html=nodes.get('#content').innerHTML;
  assert.ok(!html.includes('<script>customer'));
  assert.ok(html.includes('&lt;script&gt;customer&lt;/script&gt;'));
  assert.ok(html.includes('1.5.7')&&html.includes('1.5.8'));
  assert.ok(html.includes('<progress max="100" value="50"></progress>'));
  assert.ok(html.includes('Installing update')&&html.includes('Report is over 15 minutes old'));
  assert.equal(timers.size,1);
  await nodes.get('#installation-next').onclick();
  assert.ok(nodes.get('#content').innerHTML.includes('No installation reports yet'));
  assert.equal(timers.size,1,'Changing pages must replace the prior refresh');
  await nodes.get('#installation-refresh').onclick();
  assert.equal(timers.size,1);
  hold=true;
  const pending=nodes.get('#installation-refresh').onclick();
  assert.equal(typeof release,'function');
  const logout=nodes.get('#logout').onclick();
  assert.equal(timers.size,0,'Sign-out must stop refresh before the server acknowledges it');
  const signedOut=nodes.get('#content').innerHTML;
  release();await pending;
  assert.equal(nodes.get('#content').innerHTML,signedOut,'A response started before sign-out must never repopulate private installation data');
  assert.ok(!signedOut.includes('customer&lt;/script&gt;'));
  assert.equal(typeof releaseLogout,'function');releaseLogout();await logout;
  assert.ok(requests.some(request=>request.url.endsWith('offset=200')));
  console.log('Installation status: escaping, progress, pagination, bounded refresh and sign-out race checks passed.');
}
main().catch(error=>{console.error(error);process.exitCode=1;});
