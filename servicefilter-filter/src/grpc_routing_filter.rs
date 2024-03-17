use std::{future, sync::Arc};

use hyper::body::HttpBody;
use async_trait::async_trait;
use servicefilter_core::{filter::{ServicefilterChain, ServicefilterExchange, ServicefilterFilter}, service::{FilterConfig, ServiceConfig}};
use tonic::{client::GrpcService, body::BoxBody, Status};

pub struct GrpcRoutingFilter {
    service_config_base: Arc<ServiceConfig>,
    filter_config: FilterConfig, 
}

// TODO hold the connect by instance id
impl GrpcRoutingFilter {
    pub fn new(
        service_config_base: Arc<ServiceConfig>,
        filter_config: FilterConfig, 
    ) -> Self {
        Self{
            service_config_base,
            filter_config,
        }
    }
}

#[async_trait]
impl ServicefilterFilter for GrpcRoutingFilter {

    async fn do_filter(&self, exchange: &mut dyn ServicefilterExchange, chain: &dyn ServicefilterChain) {
        if let Some(instance) = exchange.get_service_instance() {
            let protocol = instance.get_service_protocol();
            if &"grpc".to_string() == protocol {
                // chain.do_chian(exchange).await;
                let addr = instance.get_address().clone();
                let req = exchange.get_req();
                let resp = Box::pin(async move {
                    let connect = tonic::transport::Endpoint::new(addr).unwrap().connect().await;
                    let mut channel = connect.unwrap();


                    let _ = future::poll_fn(|cx| channel.poll_ready(cx)).await;
                    let resp = channel.call(req.unwrap());
                    let resp = resp.await.map(|resp|{
                        let (parts, body) = resp.into_parts();
                        let box_body = BoxBody::new(body.map_err(|e|{
                            let status = Status::unknown(format!("translate error: {}", e));
                            status
                        }));
                        let box_resp = http::Response::from_parts(parts, box_body);
                        box_resp
                    }).map_err(|_|{unimplemented!()});

                    resp
                });
                exchange.set_resp(resp);
            }            
        }

        chain.do_chian(exchange).await;
    }
}