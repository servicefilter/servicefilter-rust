
fn main() {
    tonic_build::compile_protos("proto/control_app.proto").unwrap();
}