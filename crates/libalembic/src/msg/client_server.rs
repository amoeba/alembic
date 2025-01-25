pub enum ClientServerMessage {
    Hello(String),
    UpdateString(String),
    AppendLog(String),
    HandleSendTo(Vec<u8>),
    HandleRecvFrom(Vec<u8>),
    HandleAddTextToScroll(String),
}
