
pub fn initialize_dll() -> bool {
    if let Err(e) = on_attach() {
        unsafe {
            INIT_ERROR = Some(format!("Failed to initialize DLL: {}", e));
        }
        false
    } else {
        true
    }
}
#[allow(static_mut_refs)]
pub fn ensure_dll_state() -> &'static DllState {
    unsafe {
        DLL_STATE_INIT.call_once(|| {
            DLL_STATE = Some(DllState::new().expect("Failed to initialize DLL state."));
        });
        DLL_STATE.as_ref().unwrap()
    }
}

pub fn ensure_client() -> anyhow::Result<()> {
    let dll_state = ensure_dll_state();
    let (_tx, rx) = (dll_state.tx.clone(), dll_state.rx.clone());
    let shutdown_signal = dll_state.shutdown_signal.clone();

    dll_state.runtime.spawn(async move {
        let addr: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 5000);
        let transport = tarpc::serde_transport::tcp::connect(&addr, Json::default);
        let client: WorldClient = WorldClient::new(
            tarcp_client::Config::default(),
            transport.await.expect("Failed to connect to server"),
        )
        .spawn();

        while !shutdown_signal.load(Ordering::SeqCst) {
            match rx.lock().unwrap().try_recv() {
                Ok(msg) => {
                    match msg {
                        ClientServerMessage::HandleSendTo(vec) => {
                            if let Err(e) = client.handle_sendto(context::current(), vec).await {
                                log_event(&format!("HandleSendTo error: {}", e));
                            }
                        }
                        ClientServerMessage::HandleRecvFrom(vec) => {
                            if let Err(e) = client.handle_recvfrom(context::current(), vec).await {
                                log_event(&format!("HandleRecvFrom error: {}", e));
                            }
                        }
                    }
                }
                Err(TryRecvError::Empty) => {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
                Err(TryRecvError::Disconnected) => {
                    break;
                }
            }
        }
        
        // Clean shutdown
        log_event("Client loop shutting down");
    });

    Ok(())
}