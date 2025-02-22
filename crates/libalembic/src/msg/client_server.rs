pub enum ClientServerMessage {
    AppendLog(String),
    ClientInjected(),
    ClientEjected(),
    HandleSendTo(Vec<u8>),
    HandleRecvFrom(Vec<u8>),
    HandleAddTextToScroll(String),
}
