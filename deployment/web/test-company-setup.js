// Browser-form contract checks only; these do not replace real setup/MFA tests.
'use strict';
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const vm=require('node:vm');
const {webcrypto}=require('node:crypto');
const source=fs.readFileSync(path.join(__dirname,'../../product/management/web/app.js'),'utf8').replace(/\r\n/g,'\n');
const brand={display_name:'Default support',primary_color:'#007F82',logo_svg:'<svg></svg>',support_url:'',consent_text:'Ask before support.',support_contacts:'',shortcuts:[]};

async function fixture(){
  const nodes=new Map(),requests=[];
  class Element {
    constructor(){this.listeners={};this.children=[];this.disabled=false;this.hidden=false;this.textContent='';}
    addEventListener(type,callback){this.listeners[type]=callback;}
    querySelector(selector){
      if(!this.parts)this.parts=new Map();
      if(!this.parts.has(selector))this.parts.set(selector,new Element());
      return this.parts.get(selector);
    }
    set innerHTML(html){
      this.html=html;
      for(const match of html.matchAll(/\bid="([^"]+)"/g))nodes.set('#'+match[1],new Element());
    }
    get innerHTML(){return this.html;}
  }
  for(const id of ['content','notice','title','description','nav','logout'])nodes.set('#'+id,new Element());
  const context=vm.createContext({
    document:{querySelector:selector=>nodes.get(selector)||null,querySelectorAll:()=>[]},
    crypto:webcrypto,Uint8Array,TextDecoder,atob:value=>Buffer.from(value,'base64').toString('binary'),
    FormData:class{constructor(form){return form.values;}},
    fetch:async(url,options)=>{
      requests.push({url,options,body:options.body&&JSON.parse(options.body)});
      let value;
      if(url==='/api/v1/status')value={configured:false};
      else if(url==='/api/v1/default-branding')value=brand;
      else if(url==='/api/v1/setup'&&options.method==='POST')value={configured:true};
      else if(url==='/api/v1/profile'&&options.method==='GET'){
        const profile=requests.find(r=>r.url==='/api/v1/setup').body.profile;
        value={payload:Buffer.from(JSON.stringify(profile)).toString('base64'),signature:'contract-fixture-only'};
      }else if(url==='/api/v1/profile'&&options.method==='PUT')value={revision:2};
      else throw Error('Unexpected contract request: '+url);
      return {ok:true,json:async()=>value};
    }
  });
  assert.ok(source.endsWith('initialize();\n'));
  vm.runInContext(source.replace(/initialize\(\);\n$/,'globalThis.initialization=initialize();\n'),context);
  await context.initialization;
  return {context,nodes,requests};
}
function values(){
  const v=new FormData();
  for(const [key,value] of Object.entries({url:'https://support.example.org/',rendezvous:'id.example.org:32116',relay:'relay.example.org:32117',key:'public-transport-key',setup_token:'one-use-setup-token',username:'owner',password:'example password for form contract',code:'123456',rollout:'23',update_channel:'test',maintenance_start:'23',maintenance_end:'4',paused:'on'}))v.set(key,value);
  for(const edition of ['customer','technician']){
    for(const [key,value] of Object.entries({name:edition==='customer'?'Example customer care':'Example service desk',color:edition==='customer'?'#123456':'#654321',support:'https://example.org/'+edition,contacts:'Company help desk',consent:'Only approve a known technician.',logo:'<svg><path d="M0 0"/></svg>'}))v.set(edition+'_'+key,value);
    v.append(edition+'_shortcut_label','Company '+edition);v.append(edition+'_shortcut_url','https://example.org/help/'+edition);
  }
  return v;
}
async function submit(form,v){form.values=v;await form.listeners.submit({preventDefault(){},target:form});}

(async()=>{
  const f=await fixture();
  const html=f.nodes.get('#content').innerHTML;
  for(const name of ['customer_name','technician_name','customer_color','technician_color','customer_contacts','technician_contacts','customer_logo','technician_logo','unattended','paused','rollout','update_channel','maintenance_start','maintenance_end'])assert.ok(html.includes('name="'+name+'"'),name+' is available before setup');
  assert.ok(html.includes('&lt;svg&gt;'),'SVG source is escaped in the form');
  const v=values();await submit(f.nodes.get('#setup'),v);
  const sent=f.requests.find(r=>r.url==='/api/v1/setup');assert.ok(sent);
  assert.equal(sent.options.headers.Authorization,'Bearer one-use-setup-token');
  assert.match(sent.body.totp_secret,/^[A-Z2-7]{32}$/);
  assert.equal(sent.body.username,'owner');assert.equal(sent.body.totp_code,'123456');
  const p=sent.body.profile;
  assert.equal(p.management_url,'https://support.example.org');assert.equal(p.rendezvous,'id.example.org:32116');assert.equal(p.relay,'relay.example.org:32117');assert.equal(p.transport_public_key,'public-transport-key');
  for(const edition of ['customer','technician']){
    assert.equal(p[edition].display_name,v.get(edition+'_name'));assert.equal(p[edition].primary_color,v.get(edition+'_color'));assert.equal(p[edition].logo_svg,v.get(edition+'_logo'));assert.equal(p[edition].support_url,v.get(edition+'_support'));assert.equal(p[edition].support_contacts,v.get(edition+'_contacts'));assert.equal(p[edition].consent_text,v.get(edition+'_consent'));
    assert.deepEqual(p[edition].shortcuts,[{label:'Company '+edition,url:'https://example.org/help/'+edition}]);
  }
  assert.equal(p.allow_unattended,false);assert.equal(p.updates_paused,true);assert.equal(p.rollout_percent,23);assert.equal(p.update_channel,'test');assert.equal(p.maintenance_start_utc,23);assert.equal(p.maintenance_end_utc,4);
  assert.equal(brand.display_name,'Default support');assert.deepEqual(brand.shortcuts,[]);
  assert.ok(f.nodes.get('#login'),'Successful setup returns to MFA login');

  await vm.runInContext('screens.branding()',f.context);
  v.set('customer_name','Updated customer care');v.set('unattended','on');v.delete('paused');
  await submit(f.nodes.get('#branding'),v);
  const edited=f.requests.find(r=>r.url==='/api/v1/profile'&&r.options.method==='PUT').body;
  assert.equal(edited.customer.display_name,'Updated customer care');assert.equal(edited.technician.display_name,'Example service desk');assert.equal(edited.allow_unattended,true);assert.equal(edited.updates_paused,false);

  const invalid=await fixture(),bad=values();bad.delete('customer_shortcut_url');
  await submit(invalid.nodes.get('#setup'),bad);
  assert.equal(invalid.requests.filter(r=>r.url==='/api/v1/setup').length,0,'Incomplete shortcuts must not create a company');
  assert.match(invalid.nodes.get('#notice').textContent,/complete website shortcuts/);
  assert.equal(invalid.nodes.get('#setup').querySelector('button[type=submit]').disabled,false);
  console.log('Company setup and profile-editor form contracts passed (not server or browser acceptance).');
})().catch(error=>{console.error(error);process.exitCode=1;});
