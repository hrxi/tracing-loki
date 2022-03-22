#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PushRequest {
    #[prost(message, repeated, tag="1")]
    pub streams: ::prost::alloc::vec::Vec<StreamAdapter>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PushResponse {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryRequest {
    #[prost(string, tag="1")]
    pub selector: ::prost::alloc::string::String,
    #[prost(uint32, tag="2")]
    pub limit: u32,
    #[prost(message, optional, tag="3")]
    pub start: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(message, optional, tag="4")]
    pub end: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(enumeration="Direction", tag="5")]
    pub direction: i32,
    #[prost(string, repeated, tag="7")]
    pub shards: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(message, repeated, tag="8")]
    pub deletes: ::prost::alloc::vec::Vec<Delete>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SampleQueryRequest {
    #[prost(string, tag="1")]
    pub selector: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub start: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(message, optional, tag="3")]
    pub end: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(string, repeated, tag="4")]
    pub shards: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(message, repeated, tag="5")]
    pub deletes: ::prost::alloc::vec::Vec<Delete>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Delete {
    #[prost(string, tag="1")]
    pub selector: ::prost::alloc::string::String,
    #[prost(int64, tag="2")]
    pub start: i64,
    #[prost(int64, tag="3")]
    pub end: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryResponse {
    #[prost(message, repeated, tag="1")]
    pub streams: ::prost::alloc::vec::Vec<StreamAdapter>,
    #[prost(message, optional, tag="2")]
    pub stats: ::core::option::Option<super::stats::Ingester>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SampleQueryResponse {
    #[prost(message, repeated, tag="1")]
    pub series: ::prost::alloc::vec::Vec<Series>,
    #[prost(message, optional, tag="2")]
    pub stats: ::core::option::Option<super::stats::Ingester>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LabelRequest {
    #[prost(string, tag="1")]
    pub name: ::prost::alloc::string::String,
    /// True to fetch label values, false for fetch labels names.
    #[prost(bool, tag="2")]
    pub values: bool,
    #[prost(message, optional, tag="3")]
    pub start: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(message, optional, tag="4")]
    pub end: ::core::option::Option<::prost_types::Timestamp>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LabelResponse {
    #[prost(string, repeated, tag="1")]
    pub values: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StreamAdapter {
    #[prost(string, tag="1")]
    pub labels: ::prost::alloc::string::String,
    #[prost(message, repeated, tag="2")]
    pub entries: ::prost::alloc::vec::Vec<EntryAdapter>,
    /// hash contains the original hash of the stream.
    #[prost(uint64, tag="3")]
    pub hash: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EntryAdapter {
    #[prost(message, optional, tag="1")]
    pub timestamp: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(string, tag="2")]
    pub line: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Sample {
    #[prost(int64, tag="1")]
    pub timestamp: i64,
    #[prost(double, tag="2")]
    pub value: f64,
    #[prost(uint64, tag="3")]
    pub hash: u64,
}
/// LegacySample exists for backwards compatibility reasons and is deprecated. Do not use.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LegacySample {
    #[prost(double, tag="1")]
    pub value: f64,
    #[prost(int64, tag="2")]
    pub timestamp_ms: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Series {
    #[prost(string, tag="1")]
    pub labels: ::prost::alloc::string::String,
    #[prost(message, repeated, tag="2")]
    pub samples: ::prost::alloc::vec::Vec<Sample>,
    #[prost(uint64, tag="3")]
    pub stream_hash: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TailRequest {
    #[prost(string, tag="1")]
    pub query: ::prost::alloc::string::String,
    #[prost(uint32, tag="3")]
    pub delay_for: u32,
    #[prost(uint32, tag="4")]
    pub limit: u32,
    #[prost(message, optional, tag="5")]
    pub start: ::core::option::Option<::prost_types::Timestamp>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TailResponse {
    #[prost(message, optional, tag="1")]
    pub stream: ::core::option::Option<StreamAdapter>,
    #[prost(message, repeated, tag="2")]
    pub dropped_streams: ::prost::alloc::vec::Vec<DroppedStream>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SeriesRequest {
    #[prost(message, optional, tag="1")]
    pub start: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(message, optional, tag="2")]
    pub end: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(string, repeated, tag="3")]
    pub groups: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(string, repeated, tag="4")]
    pub shards: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SeriesResponse {
    #[prost(message, repeated, tag="1")]
    pub series: ::prost::alloc::vec::Vec<SeriesIdentifier>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SeriesIdentifier {
    #[prost(map="string, string", tag="1")]
    pub labels: ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DroppedStream {
    #[prost(message, optional, tag="1")]
    pub from: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(message, optional, tag="2")]
    pub to: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(string, tag="3")]
    pub labels: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TimeSeriesChunk {
    #[prost(string, tag="1")]
    pub from_ingester_id: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub user_id: ::prost::alloc::string::String,
    #[prost(message, repeated, tag="3")]
    pub labels: ::prost::alloc::vec::Vec<LabelPair>,
    #[prost(message, repeated, tag="4")]
    pub chunks: ::prost::alloc::vec::Vec<Chunk>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LabelPair {
    #[prost(string, tag="1")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub value: ::prost::alloc::string::String,
}
/// LegacyLabelPair exists for backwards compatibility reasons and is deprecated. Do not use.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LegacyLabelPair {
    #[prost(bytes="vec", tag="1")]
    pub name: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="2")]
    pub value: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Chunk {
    #[prost(bytes="vec", tag="1")]
    pub data: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransferChunksResponse {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TailersCountRequest {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TailersCountResponse {
    #[prost(uint32, tag="1")]
    pub count: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetChunkIDsRequest {
    #[prost(string, tag="1")]
    pub matchers: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub start: ::core::option::Option<::prost_types::Timestamp>,
    #[prost(message, optional, tag="3")]
    pub end: ::core::option::Option<::prost_types::Timestamp>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetChunkIDsResponse {
    #[prost(string, repeated, tag="1")]
    pub chunk_i_ds: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Direction {
    Forward = 0,
    Backward = 1,
}
