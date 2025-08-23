# gRpc

## Protocol Buffers

```
message AddRequest {
  int32 a = 1;
  int32 b = 2;
}
```

```rs
// Rust 구조체로 변환됨
#[derive(Clone, PartialEq, Message)]
pub struct AddRequest {
    #[prost(int32, tag="1")]
    pub a: i32,
    #[prost(int32, tag="2")] 
    pub b: i32,
}

```