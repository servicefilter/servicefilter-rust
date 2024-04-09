use std::{future::Future, pin::Pin, sync::Arc};

use servicefilter_core::{service::{ProtocolListenConfig, ProtocolServer, ServiceConfig, ServiceGenChain}, Result};
use tower::make::Shared;

pub struct ProtocolServiceGrpc {
    // service_config: Arc<ServiceConfig>,
}

impl ProtocolServiceGrpc {
    // pub fn new(service_config: Arc<ServiceConfig>,) -> Self {
    //     Self {
    //         service_config,
    //     }
    // }

    pub fn new() -> Self {
        Self {
        }
    }
}

impl ProtocolServer for ProtocolServiceGrpc {
    
    fn start(&self, service_config: Arc<ServiceConfig>, listen_config: ProtocolListenConfig, gen_chain: Box<dyn ServiceGenChain>) -> Result<Pin<Box<dyn Future<Output=()>+Send>>> {

        let addr = &listen_config.address;
        let addr = addr.parse().unwrap();
    
        let chain = chan_manager::ServiceFilterChainGrpcAgent::new(gen_chain);
    
        let h2c = h2c::H2c { s: chain };
    
        let server = hyper::Server::bind(&addr).serve(Shared::new(h2c));
        let server = Box::pin(async move {
            server.await.unwrap();
        });
        tracing::info!("{} listening on {}", service_config.service_name, addr);
        Ok(server)


    }

}

mod chan_manager {
    use std::{collections::HashMap, mem::replace, panic, sync::Arc};

    use futures::{future::{self, Ready}, FutureExt};
    use servicefilter_core::{filter::{ServiceInstance, ServicefilterExchange}, service::ServiceGenChain, X_SERVICEFILTER_TARGET_SERVICE_ID, X_SERVICEFILTER_TARGET_SERVICE_NAME};
    use tonic::{body::empty_body, codegen::BoxFuture};

    #[derive(Clone)]
    pub struct ServiceFilterChainGrpcAgent {
        gen_chain: Arc<Box<dyn ServiceGenChain>>,

    }

    impl ServiceFilterChainGrpcAgent {
        pub fn new(gen_chain: Box<dyn ServiceGenChain>) -> Self {
            Self {
                gen_chain: Arc::new(gen_chain),
            }

        }
    }

    impl tonic::codegen::Service<hyper::Request<tonic::body::BoxBody>> for ServiceFilterChainGrpcAgent
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
    
        fn poll_ready(
            &mut self,
            _: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), Self::Error>> {
            std::task::Poll::Ready(Ok(()))
        }
    
        fn call(&mut self, req: hyper::Request<tonic::body::BoxBody>) -> Self::Future {
            let gen_chain = self.gen_chain.clone();
            return Box::pin(async move {
                let headers = req.headers();
                let target_service_id_op = headers.get(X_SERVICEFILTER_TARGET_SERVICE_ID);
                let mut target_service_id = None;
                if let Some(id) = target_service_id_op {
                    target_service_id = Some(String::from(id.to_str().unwrap()));
    
                }
                let mut target_service_name = None;
                let target_service_name_op = headers.get(X_SERVICEFILTER_TARGET_SERVICE_NAME);
                if let Some(target_service_name_val) = target_service_name_op {
                    if let Ok(target_service_name_str) = target_service_name_val.to_str() {
                        target_service_name = Some(String::from(target_service_name_str));
                    }
                }
                if None == target_service_name {
                    return Ok(
                        http::Response::builder()
                            .status(200)
                            .header("grpc-status", "3")
                            .header("content-type", "application/grpc")
                            .header("x-servicefilter-error-msg", format!("not found {} header", X_SERVICEFILTER_TARGET_SERVICE_NAME))
                            .body(empty_body())
                            .unwrap(),
                    );
                } 

                // let target_service_name = headers.get(X_SERVICEFILTER_TARGET_SERVICE_NAME).unwrap().to_str().unwrap();
    
                let mut exchange = ServiceExchangeImpl::new(
                    target_service_id, 
                    String::from(target_service_name.unwrap()), 
                    HashMap::new(),
                    Some(req),
                    None,
                    None,
                );

                let chain = gen_chain.gen_chain().await;
                let result = async {
                    // AssertUnwindSafe moved to the future
                    std::panic::AssertUnwindSafe(chain.do_chian(&mut exchange)).catch_unwind().await
                }.await;

                // let chain = gen_chain.gen_chain().await;
                // chain.do_chian(&mut exchange).await;

                if result.is_err() {
                    return Ok(
                        http::Response::builder()
                            .status(200)
                            .header("grpc-status", "2")
                            .header("content-type", "application/grpc")
                            .body(empty_body())
                            .unwrap(),
                    );
                }
                // exchange.resp.unwrap()
                let resp = exchange.get_resp();
                match resp {
                    Some(resp) => {
                        // return resp.await;

                        // let result = async {resp}.catch_unwind().await;
                        // // println!("{:?}", result);
                        // return Ok(result.unwrap());
                        let result_handle = async {
                            // AssertUnwindSafe moved to the future
                            let data = std::panic::AssertUnwindSafe(resp).catch_unwind().await;
                            return data;
                        }.await;
                        if result_handle.is_err() {
                            return Ok(
                                http::Response::builder()
                                    .status(200)
                                    .header("grpc-status", "2")
                                    .header("content-type", "application/grpc")
                                    .body(empty_body())
                                    .unwrap(),
                            );
                        }
                        return result_handle.unwrap();
                        // return resp.await;
                    },
                    None => {
                        return Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        );
                    },
                }
            });
        }   
    }

    impl tonic::server::NamedService for ServiceFilterChainGrpcAgent {
        const NAME: &'static str = "servicefilter-chain-manager";
    }

    struct ServiceExchangeImpl {
        target_service_id: Option<String>,
        target_service_name: String,
        attributes: HashMap<String, String>,
        // TODO change to common req and resp like req{header:{key:value}, body: stream<Bytes>} resp{header:{key:value}, body: Future<Result<stream<Bytes>>, Status>}
        req: Option<http::Request<tonic::body::BoxBody>>,
        resp: Option<tonic::codegen::BoxFuture<http::Response<tonic::body::BoxBody>, std::convert::Infallible>>,
        service_instance: Option<Box<dyn ServiceInstance>>,
    }
    
    impl ServiceExchangeImpl {
        pub fn new(target_service_id: Option<String>,
            target_service_name: String,
            attributes: HashMap<String, String>,
            req: Option<http::Request<tonic::body::BoxBody>>,
            resp: Option<tonic::codegen::BoxFuture<http::Response<tonic::body::BoxBody>, std::convert::Infallible>>,
            service_instance: Option<Box<dyn ServiceInstance>>,) -> Self {
            Self { target_service_id, target_service_name, attributes, req, resp, service_instance }
        }
    }
    
    impl ServicefilterExchange for ServiceExchangeImpl {
    
        fn get_target_service_id(&self) -> &Option<String> {
            return &self.target_service_id;
        }
    
        fn get_target_service_name(&self) -> &String {
            return &self.target_service_name;
        }
    
        fn get_req(&mut self) -> Option<http::Request<tonic::body::BoxBody>> {
            replace(&mut self.req, None)
        }
    
        fn get_resp(&mut self) -> Option<tonic::codegen::BoxFuture<http::Response<tonic::body::BoxBody>, std::convert::Infallible>> {
            replace(&mut self.resp, None)
        }
    
        fn set_resp(&mut self, resp : tonic::codegen::BoxFuture<http::Response<tonic::body::BoxBody>, std::convert::Infallible>,) {
            self.resp = Some(resp);
        }
    
        fn get_attributes(&self) -> &HashMap<String, String> {
            return &self.attributes;
        }
    
        fn get_attribute(&self, key: &String) -> Option<&String> {
            return self.attributes.get(key);
        }
    
        fn put_attribute(&mut self, key: &String, val: &String) {
            self.attributes.insert(String::from(key), String::from(val));
        }
    
        fn get_service_instance(&self) -> &Option<Box<dyn ServiceInstance>> {
            return &self.service_instance;
        }
    
        fn set_service_instance(&mut self, service_instance: Box<dyn ServiceInstance>,) {
            self.service_instance = Some(service_instance);
        }
    }

}

pub mod h2c {
    use std::convert::Infallible;

    use http::Response;
    use tonic::{codegen::Service, codegen::BoxFuture, body::BoxBody, transport::NamedService, Status};
    use hyper::body::HttpBody;

    #[derive(Clone)]
    pub struct H2c<S> {
        pub s: S,
    }

    impl<S> Service<hyper::Request<hyper::Body>> for H2c<S>
    where
        S: Service<hyper::Request<tonic::body::BoxBody>, Response = Response<BoxBody>, Error = Infallible>
        + NamedService
            + Clone
            + Send
            + 'static,
        S::Future: Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;

        fn poll_ready(
            &mut self,
            _: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), Self::Error>> {
            std::task::Poll::Ready(Ok(()))
        }

        fn call(&mut self, req: hyper::Request<hyper::Body>) -> Self::Future {
            let mut svc = self.s.clone();
            let (parts, body) = req.into_parts();
            let box_body = BoxBody::new(body.map_err(|e|{
                let status = Status::unknown(format!("translate error: {}", e));
                status
            }));

            let req = hyper::Request::from_parts(parts, box_body);
            let resp= svc.call(req);
            Box::pin(resp)   
        }
    }
}
