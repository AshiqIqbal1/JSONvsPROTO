fn main() {
    prost_build::compile_protos(&["../schema/user.proto"], &["../schema/"]).unwrap();
}
