

fn main() {
    tonic_build::compile_protos("proto/redisoperate.proto").unwrap();
}