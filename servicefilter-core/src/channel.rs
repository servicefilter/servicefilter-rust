use async_trait::async_trait;
use tonic::body::BoxBody;
use tower::Service;

// pub type FilterReqChannel = dyn Service<http::Request<BoxBody>, Response = http::Response<tonic::body::BoxBody>, Error = std::convert::Infallible, Future = tonic::codegen::BoxFuture<http::Response<tonic::body::BoxBody>, std::convert::Infallible>>;

pub type FilterReqChannel = dyn Service<http::Request<BoxBody>, Response = http::Response<tonic::body::BoxBody>, Error = std::convert::Infallible, Future = tonic::codegen::BoxFuture<http::Response<tonic::body::BoxBody>, std::convert::Infallible>> + Send;

#[async_trait]
pub trait FilterReqChannelGen: Send + Sync {
    async fn gen_channel(&self) -> Box<FilterReqChannel>;
}