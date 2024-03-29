use std::{pin::Pin, sync::Arc};

use async_trait::async_trait;
use http_body::combinators::UnsyncBoxBody;
use servicefilter_core::{filter::{ServicefilterChain, ServicefilterExchange, ServicefilterFilter}, service::{FilterConfig, ServiceConfig}, X_SERVICEFILTER_TARGET_SERVICE_NAME};
use tonic::{body::empty_body, metadata::MetadataValue, transport::Server, IntoRequest, Response, Status};

use self::hello_world::{redis_operate_service_client::RedisOperateServiceClient, simple_server::{Simple, SimpleServer}, GetRequest, HelloReply, HelloRequest, RedisGetReply, RedisGetRequest};

use hyper::{body::HttpBody, service::Service};


pub mod hello_world {
    tonic::include_proto!("servicefilter.helloworld.mock");
    tonic::include_proto!("servicefilter.sdk.redis");
}

pub struct ServicefilterLibMockServerFilter {
    service_base_load_info: Arc<ServiceConfig>,
    filter_config: FilterConfig,
    channel_gen : Arc<Box<dyn servicefilter_core::channel::FilterReqChannelGen>>,
}

impl ServicefilterLibMockServerFilter {
    pub fn new(
        service_base_load_info: Arc<ServiceConfig>,
        filter_config: FilterConfig,
        channel_gen : Arc<Box<dyn servicefilter_core::channel::FilterReqChannelGen>>,
    ) -> Self {
        Self {
            service_base_load_info,
            filter_config,
            channel_gen,
        }
    }
}

#[async_trait]
impl ServicefilterFilter for ServicefilterLibMockServerFilter {
    
    async fn do_filter(&self, exchange: &mut dyn ServicefilterExchange, chain: &dyn ServicefilterChain) {

        let greeter = MyGreeter{channel_gen: self.channel_gen.clone()};
        let server = SimpleServer::new(greeter);
        let mut svc = Server::builder().add_service(server).into_router();
        let req = exchange.get_req();
        
        let req = req.unwrap();
        let (parts,box_body) = req.into_parts();
        let stream_body = futures::stream::unfold(box_body, |mut body| async {
            
            let data = body.data().await;
            match data {
                Some(Ok(bytes)) => Some((Ok::<_, hyper::Error>(bytes), body)),
                // TODO here bug
                Some(Err(e)) => None,
                // Some(Err(e)) => Some((Err::<_, hyper::Error>(hyper::Error::from(e), body))),
                None => None,
            }
        });
        let hyper_body = hyper::Body::wrap_stream(stream_body);
        let req = hyper::Request::from_parts(parts, hyper_body);
        let hyper_resp = svc.call(req);

        let resp = Box::pin(async move {
            let result_resp = hyper_resp.await;
            match result_resp {
                Ok(resp) => {
                    let(parts, box_body) = resp.into_parts();
                    let body = UnsyncBoxBody::new(box_body.map_err(|e|{
                        //TODO bug
                            return Status::new(tonic::Code::Unknown, "convert error");
                        }));
                    
                    return Ok(hyper::Response::from_parts(parts, body));
                },
                Err(e) => { 
                    return Err(e);
                },
            }
        });
        exchange.set_resp(resp);
        
        chain.do_chian(exchange).await;
    }
}

pub struct MyGreeter {
    channel_gen : Arc<Box<dyn servicefilter_core::channel::FilterReqChannelGen>>,
}

#[async_trait]
impl Simple for MyGreeter {

    async fn say_hello(
        &self,
        request: tonic::Request<HelloRequest>,
    ) -> std::result::Result<tonic::Response<HelloReply>, tonic::Status> {

        return Ok(Response::new(HelloReply{message: String::from("hello world")}));
    }

    async fn redis_get(
        &self,
        request: tonic::Request<RedisGetRequest>,
    ) -> std::result::Result<tonic::Response<RedisGetReply>, tonic::Status> {
        let channel = self.channel_gen.gen_channel().await;

        let req = GetRequest{key: request.into_inner().key};
        let mut req = req.into_request();
        let service_name: MetadataValue<_> = "redis-example".parse().unwrap();
        req.metadata_mut().insert(X_SERVICEFILTER_TARGET_SERVICE_NAME,service_name);

        let mut client = RedisOperateServiceClient::new(channel);
        let resp = client.get(req).await.unwrap();
        let result = resp.into_inner();
        let mut resp_valalue = "".to_string();
        if let Some(val) = result.result {
            if let hello_world::get_response::Result::Value(resp_val) = val {
                resp_valalue = resp_val;
            }
        }
        return Ok(Response::new(RedisGetReply{value: resp_valalue}));
    }
}