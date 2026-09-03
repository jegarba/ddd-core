/// Espera SIGINT (Ctrl-C) o SIGTERM — usar con
/// `axum::serve(listener, app).with_graceful_shutdown(signal())`.
pub async fn signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.expect("no se pudo instalar el handler de ctrl-c");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("no se pudo instalar el handler de SIGTERM")
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
