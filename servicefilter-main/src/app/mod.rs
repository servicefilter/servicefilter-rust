pub(crate) mod control;

use std::{collections::HashMap, future::Future, sync::Arc};

use servicefilter_core::{service::ServiceConfigOpreate, Result};
use servicefilter_load::LoadFactory;
use tokio::sync::{broadcast, mpsc, RwLock};

use crate::{cmd::{cli::Cli, config::{ServicefilterAppConfig, ServicefilterServiceConfig}}, service::ServiceRunHandler};

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

    pub services: Arc<RwLock<HashMap<String, ServiceRunHandler>>>,

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
        let saved_config = &self.config;
        let config = ServicefilterAppConfig::from_cli(&self.cli).await?;
        let service_configs = &config.services;
        let mut write = self.services.write().await;

        for service_config in service_configs {
            let service_op = write.get(&service_config.service_id);
            match service_op {
                None => {
                    let service_operate = Self::build_handler(saved_config, service_config, self.load_factory.clone()).await;
            
                    write.insert(String::from(&service_config.service_id), service_operate);
                },
                Some(handler) => {
                    // TODO timeout？
                    handler.reload(service_config.clone()).await;
                }
            }
        }

        let to_remove_keys: Vec<_> = write.keys()
            .filter(|&key| !service_configs.iter().any(|config| &config.service_id == key))
            .cloned()
            .collect();
        
        for key in to_remove_keys {
            let handler_op = write.remove(&key);
            if let Some(handler) = handler_op {
                handler.stop().await;
            }
        }

        drop(write);
        return Ok(());
    }

    async fn start_service(
        config: &ServicefilterAppConfig,
        // service_configs: &Vec<ServicefilterServiceConfig>, 
        services: Arc<RwLock<HashMap<String, ServiceRunHandler>>>,
        // TODO graceful shutdown
        notify_shutdown: broadcast::Sender<()>,
        shutdown_complete_tx: mpsc::Sender<()>,
        load_factory: Arc<LoadFactory>,) {
        let service_configs = &config.services;
        let mut write = services.write().await;
        for service_config in service_configs {

            let service_operate = Self::build_handler(config, service_config, load_factory.clone()).await;
            
            write.insert(String::from(&service_config.service_id), service_operate);
        }
        drop(write);
    }

    async fn build_handler(config: &ServicefilterAppConfig, service_config: &ServicefilterServiceConfig, load_factory: Arc<LoadFactory>,) -> ServiceRunHandler {
        let mut service_run_handler = ServiceRunHandler::new(
            config.app_id.clone(),
            service_config.clone(),
            load_factory.clone(),
        );
        
        let _ = service_run_handler.run().await;
        return service_run_handler;
    }

}