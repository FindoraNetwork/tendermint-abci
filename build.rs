fn main() {
    protobuf_codegen::Codegen::new()
        .pure()
        .include("protobuf")
        .inputs(&[
            "protobuf/third_party/proto/gogoproto/gogo.proto",
            "protobuf/crypto/merkle/merkle.proto",
            "protobuf/libs/kv/types.proto",
            "protobuf/abci.proto",
        ])
        .cargo_out_dir("generated_with_pure")
        .run_from_script();
}
