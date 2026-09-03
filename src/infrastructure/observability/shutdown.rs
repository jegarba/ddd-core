/// Waits for SIGINT (Ctrl-C) or SIGTERM — use with
/// `axum::serve(listener, app).with_graceful_shutdown(signal())`.
pub async fn signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.expect("failed to install ctrl-c handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {}
        _ = terminate => {}
    }
}
