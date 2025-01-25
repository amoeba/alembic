pub enum ClientServerMessage {
    Hello(String),
    UpdateString(String),
    AppendLog(String),
    SendTo(Vec<u8>),
    RecvFrom(Vec<u8>),
    AddTextToScroll(String),
}
