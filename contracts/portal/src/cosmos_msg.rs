use prost;
use prost::Message;

#[derive(Clone, PartialEq, Message)]
pub struct MsgInstantiateContract2 {
    /// Sender is the that actor that signed the messages
    #[prost(string, tag = "1")]
    pub sender: ::prost::alloc::string::String,
    /// Admin is an optional address that can execute migrations
    #[prost(string, tag = "2")]
    pub admin: ::prost::alloc::string::String,
    /// CodeID is the reference to the stored WASM code
    #[prost(uint64, tag = "3")]
    pub code_id: u64,
    /// Label is optional metadata to be stored with a contract instance.
    #[prost(string, tag = "4")]
    pub label: ::prost::alloc::string::String,
    /// Msg json encoded message to be passed to the contract on instantiation
    #[prost(bytes = "vec", tag = "5")]
    pub msg: ::prost::alloc::vec::Vec<u8>,
    /// Funds coins that are transferred to the contract on instantiation
    #[prost(message, repeated, tag = "6")]
    pub funds: ::prost::alloc::vec::Vec<CosmosCoin>,
    /// Salt is an arbitrary value provided by the sender. Size can be 1 to 64.
    #[prost(bytes = "vec", tag = "7")]
    pub salt: ::prost::alloc::vec::Vec<u8>,
    /// FixMsg include the msg value into the hash for the predictable address.
    /// Default is false
    #[prost(bool, tag = "8")]
    pub fix_msg: bool,
}

#[derive(Clone, PartialEq, Message)]
pub struct CosmosCoin {
    #[prost(string, tag = "1")]
    pub denom: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub amount: ::prost::alloc::string::String,
}