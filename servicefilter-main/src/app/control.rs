use std::sync::Arc;

use servicefilter_core::Result;
use servicefilter_load::LoadFactory;
use tonic::transport::Server;


use self::control_app_grpc::app_grpc::servicefilter_manager_service_server::ServicefilterManagerServiceServer;

use super::ServicefilterApp;

pub(crate) struct ControlApp {
    app: Arc<ServicefilterApp>,
    load_factory: Arc<LoadFactory>,
}

impl ControlApp {
    pub fn new(app: Arc<ServicefilterApp>, load_factory: Arc<LoadFactory>,) -> Self {
        Self {  
            app,
            load_factory,
        }
    }

    pub async fn start(&self) -> Result<()> {
        let control = control_app_grpc::ControlAppGrpc::new(self.app.clone());
        let server = ServicefilterManagerServiceServer::new(control);
        let control_listen = &self.app.config.control_listen;

        let addr = &control_listen.address;
        let addr = addr.parse().unwrap();

        tracing::info!("control listening on {}", addr);
        Server::builder().add_service(server).serve(addr).await?;
        return Ok(());
    }
}

mod control_app_grpc {
    use std::sync::Arc;

    use crate::app::ServicefilterApp;

    use self::app_grpc::{servicefilter_manager_service_server::ServicefilterManagerService, HealthRequest, HealthResponse, ReloadRequest, ReloadResponse};

    pub mod app_grpc {
        tonic::include_proto!("servicefilter.control.app.proto");
    }

    pub struct ControlAppGrpc {
        app: Arc<ServicefilterApp>,
    }

    impl ControlAppGrpc {
        pub fn new(app: Arc<ServicefilterApp>,) -> Self {
            Self {
                app,
            }
        }
    }

    #[tonic::async_trait]
    impl ServicefilterManagerService for ControlAppGrpc {

        async fn health(
            &self,
            request: tonic::Request<HealthRequest>,
        ) -> std::result::Result<tonic::Response<HealthResponse>, tonic::Status> {

            Ok(tonic::Response::new(HealthResponse{message: String::from("PONG")}))
        }

        async fn reload(
            &self,
            request: tonic::Request<ReloadRequest>,
        ) -> std::result::Result<tonic::Response<ReloadResponse>, tonic::Status> {
            let _ = self.app.reload().await;
            Ok(tonic::Response::new(ReloadResponse{}))
        }
    }
}