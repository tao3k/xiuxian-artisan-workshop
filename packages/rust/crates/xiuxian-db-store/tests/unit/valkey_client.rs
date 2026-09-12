use super::{ValkeyClient, ValkeyStoreConfig, ValkeyStoreError};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use tokio::io::AsyncReadExt;

async fn connection() -> Result<
    (
        Arc<redis::aio::MultiplexedConnection>,
        tokio::io::DuplexStream,
    ),
    Box<dyn std::error::Error>,
> {
    let (client, server) = tokio::io::duplex(4096);
    let info = redis::RedisConnectionInfo::default().set_skip_set_lib_name();
    let (connection, driver) = redis::aio::MultiplexedConnection::new_with_config(
        &info,
        client,
        redis::AsyncConnectionConfig::default(),
    )
    .await?;
    tokio::spawn(driver);
    Ok((Arc::new(connection), server))
}

#[tokio::test]
async fn applied_mutation_with_lost_response_is_not_replayed()
-> Result<(), Box<dyn std::error::Error>> {
    let client = ValkeyClient::new(ValkeyStoreConfig::new("redis://127.0.0.1:1")?);
    let (conn, mut server) = connection().await?;
    *client.connection.write().await = Some(conn);
    let effects = Arc::new(AtomicUsize::new(0));
    let applied = Arc::clone(&effects);
    let mut command = redis::cmd("INCR");
    command.arg("counter");
    let wire = command.get_packed_command();
    let server_task = tokio::spawn(async move {
        let mut received = vec![0; wire.len()];
        server.read_exact(&mut received).await?;
        assert_eq!(received, wire);
        applied.fetch_add(1, Ordering::SeqCst);
        drop(server);
        Ok::<(), std::io::Error>(())
    });
    let attempts = AtomicUsize::new(0);
    let result: Result<i64, ValkeyStoreError> = client
        .run_command("test_incr", || {
            attempts.fetch_add(1, Ordering::SeqCst);
            command.clone()
        })
        .await;
    server_task.await??;
    assert!(matches!(
        result,
        Err(ValkeyStoreError::OutcomeUnknown { .. })
    ));
    assert_eq!(attempts.load(Ordering::SeqCst), 1);
    assert_eq!(effects.load(Ordering::SeqCst), 1);
    Ok(())
}

#[tokio::test]
async fn stale_failure_cannot_invalidate_replacement() -> Result<(), Box<dyn std::error::Error>> {
    let client = ValkeyClient::new(ValkeyStoreConfig::new("redis://127.0.0.1:1")?);
    let (old, _old_server) = connection().await?;
    let (new, _new_server) = connection().await?;
    *client.connection.write().await = Some(Arc::clone(&new));
    client.invalidate_connection(&old).await;
    assert!(
        client
            .connection
            .read()
            .await
            .as_ref()
            .is_some_and(|current| Arc::ptr_eq(current, &new))
    );
    client.invalidate_connection(&new).await;
    assert!(client.connection.read().await.is_none());
    Ok(())
}
