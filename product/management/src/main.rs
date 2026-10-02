mod service;
mod backup;
mod network;
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
        let operation=args[1].clone();let extra=args[3..].to_vec();
        tokio::task::spawn_blocking(move || ->Result<()> {
            let mut inputs=std::collections::HashMap::new();
            anyhow::ensure!(extra.len()%2==0,"Backup options require explicit paths");
            for pair in extra.chunks_exact(2){
                anyhow::ensure!(operation=="backup" && matches!(pair[0].as_str(),"--transport-directory"|"--deployment-env"|"--tls-identity"|"--tls-directory"),"Unknown backup option");
                anyhow::ensure!(inputs.insert(pair[0].clone(),PathBuf::from(&pair[1])).is_none(),"Duplicate backup option");
            }
            if operation=="backup" {
                backup::export_deployment(&data,&path,&passphrase,inputs.get("--transport-directory").map(|p|p.as_path()),inputs.get("--deployment-env").map(|p|p.as_path()),inputs.get("--tls-identity").map(|p|p.as_path()),inputs.get("--tls-directory").map(|p|p.as_path()))
            }else{backup::restore(&data,&path,&passphrase)}
        }).await??;
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
