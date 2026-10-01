mod service;
mod backup;
#[cfg(test)]
mod tests;
use anyhow::{Context, Result};
use axum::Router;
use service::Store;
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
#[cfg(windows)]
mod windows;

fn main() -> Result<()> {
    #[cfg(windows)]
    if std::env::args().any(|a|a=="--service") {return windows::dispatch();}
    // This is the executable entrypoint, not a helper within an existing runtime.
    let runtime=tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        let (sender,receiver)=tokio::sync::oneshot::channel();
        tokio::spawn(async move {if tokio::signal::ctrl_c().await.is_ok(){let _=sender.send(());}});
        run(receiver).await
    })
}

async fn run(stop:tokio::sync::oneshot::Receiver<()>)->Result<()> {
    let data = PathBuf::from(std::env::var("SWAN_DATA_DIR").unwrap_or_else(|_| "swan-data".into()));
    let args:Vec<String>=std::env::args().collect();
    if matches!(args.get(1).map(String::as_str),Some("backup"|"restore")) {
        let path=PathBuf::from(args.get(2).context("Usage: swan-management backup|restore FILE")?);
        let passphrase=std::env::var("SWAN_BACKUP_PASSPHRASE").context("Set SWAN_BACKUP_PASSPHRASE for this process")?;
        if args[1]=="backup" {backup::export(&data,&path,&passphrase)?;} else {backup::restore(&data,&path,&passphrase)?;}
        println!("Encrypted {} completed.",args[1]);return Ok(());
    }
    let store = Arc::new(Store::open(&data)?);
    let address: SocketAddr = std::env::var("SWAN_LISTEN").unwrap_or_else(|_| "127.0.0.1:8080".into()).parse()?;
    let app: Router = service::router(store);
    let listener = tokio::net::TcpListener::bind(address).await.context("Bind management listener")?;
    println!("Swan Remote Support management listening on {address}; secrets stored under {}",data.display());
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).with_graceful_shutdown(async {let _=stop.await;}).await?;
    Ok(())
}
