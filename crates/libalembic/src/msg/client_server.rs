pub enum ClientServerMessage {
    AppendLog(String),
    HandleSendTo(Vec<u8>),
    HandleRecvFrom(Vec<u8>),
    HandleAddTextToScroll(String),
}
