

fn main() {
    tonic_build::compile_protos("proto/helloworld.proto").unwrap();
    tonic_build::compile_protos("proto/redisoperate.proto").unwrap();
}