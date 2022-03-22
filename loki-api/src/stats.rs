/// Result contains LogQL query statistics.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Result {
    #[prost(message, optional, tag="1")]
    pub summary: ::core::option::Option<Summary>,
    #[prost(message, optional, tag="2")]
    pub querier: ::core::option::Option<Querier>,
    #[prost(message, optional, tag="3")]
    pub ingester: ::core::option::Option<Ingester>,
}
/// Summary is the summary of a query statistics.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Summary {
    /// Total bytes processed per second.
    #[prost(int64, tag="1")]
    pub bytes_processed_per_second: i64,
    /// Total lines processed per second.
    #[prost(int64, tag="2")]
    pub lines_processed_per_second: i64,
    /// Total bytes processed.
    #[prost(int64, tag="3")]
    pub total_bytes_processed: i64,
    /// Total lines processed.
    #[prost(int64, tag="4")]
    pub total_lines_processed: i64,
    /// Execution time in seconds.
    /// In addition to internal calculations this is also returned by the HTTP API.
    /// Grafana expects time values to be returned in seconds as float.
    #[prost(double, tag="5")]
    pub exec_time: f64,
    /// Queue time in seconds.
    /// In addition to internal calculations this is also returned by the HTTP API.
    /// Grafana expects time values to be returned in seconds as float.
    #[prost(double, tag="6")]
    pub queue_time: f64,
    /// Total of subqueries created to fulfill this query.
    #[prost(int64, tag="7")]
    pub subqueries: i64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Querier {
    #[prost(message, optional, tag="1")]
    pub store: ::core::option::Option<Store>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Ingester {
    /// Total ingester reached for this query.
    #[prost(int32, tag="1")]
    pub total_reached: i32,
    /// Total of chunks matched by the query from ingesters
    #[prost(int64, tag="2")]
    pub total_chunks_matched: i64,
    /// Total of batches sent from ingesters.
    #[prost(int64, tag="3")]
    pub total_batches: i64,
    /// Total lines sent by ingesters.
    #[prost(int64, tag="4")]
    pub total_lines_sent: i64,
    #[prost(message, optional, tag="5")]
    pub store: ::core::option::Option<Store>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Store {
    /// The total of chunk reference fetched from index.
    #[prost(int64, tag="1")]
    pub total_chunks_ref: i64,
    /// Total number of chunks fetched.
    #[prost(int64, tag="2")]
    pub total_chunks_downloaded: i64,
    /// Time spent fetching chunks in nanoseconds.
    #[prost(int64, tag="3")]
    pub chunks_download_time: i64,
    #[prost(message, optional, tag="4")]
    pub chunk: ::core::option::Option<Chunk>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Chunk {
    /// Total bytes processed but was already in memory. (found in the headchunk)
    #[prost(int64, tag="4")]
    pub head_chunk_bytes: i64,
    /// Total lines processed but was already in memory. (found in the headchunk)
    #[prost(int64, tag="5")]
    pub head_chunk_lines: i64,
    /// Total bytes decompressed and processed from chunks.
    #[prost(int64, tag="6")]
    pub decompressed_bytes: i64,
    /// Total lines decompressed and processed from chunks.
    #[prost(int64, tag="7")]
    pub decompressed_lines: i64,
    /// Total bytes of compressed chunks (blocks) processed.
    #[prost(int64, tag="8")]
    pub compressed_bytes: i64,
    /// Total duplicates found while processing.
    #[prost(int64, tag="9")]
    pub total_duplicates: i64,
}
