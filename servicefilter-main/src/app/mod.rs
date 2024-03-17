pub(crate) mod control;

use std::{collections::HashMap, future::Future, sync::Arc};

use servicefilter_core::{service::ServiceConfigOpreate, Result};
use servicefilter_load::LoadFactory;
use tokio::sync::{broadcast, mpsc, RwLock};

use crate::{cmd::{cli::Cli, config::{ServicefilterAppConfig, ServicefilterServiceConfig}}, service::{ServiceOperateHandler, ServiceRunHandler}};

use self::control::ControlApp;


pub(crate) async fn start(cli: Cli, shutdown: impl Future) -> Result<()> {
    let config = ServicefilterAppConfig::from_cli(&cli).await?;
    tracing::debug!("config: {:?}", config);
    let (config_change_tx, mut config_change_rx) = mpsc::channel(1);
    cli.listen_config_change(config_change_tx);

    let (notify_shutdown, _) = broadcast::channel(1);
    let (shutdown_complete_tx, mut shutdown_complete_rx) = mpsc::channel(1);

    let load_factory = Arc::new(LoadFactory::new(config.lib_load_path.clone()));
    
    let app = ServicefilterApp {
        cli,
        config,
        notify_shutdown: notify_shutdown.clone(),
        shutdown_complete_tx: shutdown_complete_tx.clone(),
        services: Default::default(),
        load_factory: load_factory.clone(),
    };
    let app = Arc::new(app);

    // reload signal 
    let config_change_shutdown_sign = notify_shutdown.clone();
    let app_sign = app.clone();
    tokio::spawn(async move {
        let mut sub = config_change_shutdown_sign.subscribe();
        loop {
            tokio::select! {
                _ = config_change_rx.recv() => {
                    let _ = app_sign.reload().await;
                }
                _ = sub.recv() => {
                    break;
                }
            }
        }
    });

    // control 
    let control_app = ControlApp::new(app.clone(), load_factory.clone());
    
    let _ = app.start().await;

    tokio::select! {
        _ = control_app.start() => {}
        _ = shutdown => {}
    }

    drop(notify_shutdown);
    drop(shutdown_complete_tx);

    // config_change_handler.abort();
    drop(control_app);
    drop(app);
    
    let _ = shutdown_complete_rx.recv().await;

    return Ok(());
}

pub struct ServicefilterApp {

    pub cli: Cli,

    pub config: ServicefilterAppConfig,

    notify_shutdown: broadcast::Sender<()>,

    shutdown_complete_tx: mpsc::Sender<()>,

    pub services: Arc<RwLock<HashMap<String, ServiceOperateHandler>>>,

    load_factory: Arc<LoadFactory>,
}


impl ServicefilterApp {
    
    async fn start(&self) -> Result<()> {
        // let service_configs = &self.config.services;
        Self::start_service(&self.config, 
            // service_configs, 
            self.services.clone(), 
            self.notify_shutdown.clone(), 
            self.shutdown_complete_tx.clone(),
        self.load_factory.clone()).await;
        
        return Ok(());
    }

    pub async fn reload(&self, ) -> Result<()> {
        // panic!("hello");
        return Ok(());
    }

    async fn start_service(
        config: &ServicefilterAppConfig,
        // service_configs: &Vec<ServicefilterServiceConfig>, 
        services: Arc<RwLock<HashMap<String, ServiceOperateHandler>>>,
        // TODO graceful shutdown
        notify_shutdown: broadcast::Sender<()>,
        shutdown_complete_tx: mpsc::Sender<()>,
        load_factory: Arc<LoadFactory>,) {
        let service_configs = &config.services;
        let mut write = services.write().await;
        for service_config in service_configs {
            
            let mut config_operate = ServiceConfigOpreate {
                config_modify: Box::new(|_|{}),
                config_extract: Box::new(||{HashMap::new()}),
                config_check: Box::new(|_|{}),
            };
            let config_operate_mut = &mut config_operate;
            
            let service_run_handler = ServiceRunHandler::new(
                config.app_id.clone(),
                service_config.clone(),
                config_operate_mut,
            );

            let load_factory = load_factory.clone();
            let service_task = tokio::spawn(async move {
                tokio::select! {
                    _ = service_run_handler.run(load_factory) => {}
                }
            });

            let service_operate = ServiceOperateHandler::new(service_task, config_operate,);
            
            write.insert(String::from(&service_config.service_name), service_operate);
        }
        drop(write);
    }
}