//! Explicit live-server comparative probe; never silently passes without a server.

use std::{
    error::Error,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use xiuxian_db_store::{
    ValkeyClient, ValkeyQueueEntryId, ValkeyQueueKeys, ValkeyReadPolicy, ValkeyReadSession,
    ValkeyStoreConfig,
};

#[tokio::test]
#[ignore = "requires VALKEY_URL and a dedicated release-profile measurement run"]
async fn valkey_observation_release_probe() -> Result<(), Box<dyn Error>> {
    let url = std::env::var("VALKEY_URL")?;
    let mut connection = redis::Client::open(url.as_str())?
        .get_multiplexed_async_connection()
        .await?;
    let namespace = format!(
        "was-probe:{}:{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    );
    let keys = ValkeyQueueKeys::new(
        format!("{namespace}:pending"),
        format!("{namespace}:lease"),
        format!("{namespace}:payload:"),
        format!("{namespace}:lease:"),
    )?;
    let client = ValkeyClient::new(ValkeyStoreConfig::new(url)?);
    println!(
        "profile={} os={} samples=30 payload_bytes=256 modes=serial,pipeline32",
        if cfg!(debug_assertions) {
            "debug-NOT-PERFORMANCE-ACCEPTANCE"
        } else {
            "release"
        },
        std::env::consts::OS
    );
    for count in [100, 1000] {
        let entries: Vec<_> = (0..count)
            .map(|index| ValkeyQueueEntryId::new(index.to_string()))
            .collect::<Result<_, _>>()?;
        for chunk in entries.chunks(32) {
            let mut seed = redis::pipe();
            for entry in chunk {
                seed.cmd("HSET")
                    .arg(keys.payload_key(entry))
                    .arg("payload")
                    .arg(format!("{:0>256}", entry.as_str()))
                    .ignore();
                seed.cmd("EXPIRE")
                    .arg(keys.payload_key(entry))
                    .arg(300)
                    .ignore();
            }
            let _: () = seed.query_async(&mut connection).await?;
        }
        let mode = std::env::var("WAS_PROBE_MODE").unwrap_or_else(|_| "both".into());
        if !matches!(mode.as_str(), "both" | "serial" | "pipeline32") {
            return Err("WAS_PROBE_MODE must be both, serial, or pipeline32".into());
        }
        for pipelined in [false, true] {
            if mode == "both" || mode == if pipelined { "pipeline32" } else { "serial" } {
                measure(&client, &mut connection, &keys, &entries, pipelined).await?;
            }
        }
        let owned_keys: Vec<_> = entries
            .iter()
            .map(|entry| keys.payload_key(entry))
            .collect();
        let _: usize = redis::cmd("DEL")
            .arg(&owned_keys)
            .query_async(&mut connection)
            .await?;
    }
    Ok(())
}

async fn measure(
    client: &ValkeyClient,
    connection: &mut redis::aio::MultiplexedConnection,
    keys: &ValkeyQueueKeys,
    entries: &[ValkeyQueueEntryId],
    pipelined: bool,
) -> Result<(), Box<dyn Error>> {
    let count = entries.len();
    let mut micros = Vec::new();
    let mut submissions = 0;
    let mut admitted_bytes = 0;
    let before: String = redis::cmd("INFO")
        .arg("cpu")
        .query_async(&mut *connection)
        .await?;
    for sample in 0..33 {
        let reader = ValkeyReadSession::new(
            client.clone(),
            ValkeyReadPolicy {
                timeout: Duration::from_secs(30),
                ..Default::default()
            },
        );
        let started = Instant::now();
        let values = if pipelined {
            pipelined_payloads(&reader, keys, entries).await?
        } else {
            let mut values = Vec::with_capacity(count);
            for entry in entries {
                values.push(reader.payload(keys, entry).await?);
            }
            values
        };
        let elapsed = started.elapsed();
        assert_eq!(values.len(), count);
        assert!(values.iter().zip(entries).all(|(value, entry)| {
            value.as_deref() == Some(format!("{:0>256}", entry.as_str()).as_str())
        }));
        let receipt = reader.finish()?;
        assert_eq!(receipt.commands, count);
        if sample >= 3 {
            micros.push(elapsed.as_micros());
        }
        submissions = receipt.commands;
        admitted_bytes = receipt.bytes;
    }
    let after: String = redis::cmd("INFO")
        .arg("cpu")
        .query_async(&mut *connection)
        .await?;
    let memory: String = redis::cmd("INFO")
        .arg("memory")
        .query_async(&mut *connection)
        .await?;
    micros.sort_unstable();
    println!(
        "keys={count} mode={} p50_us={} p95_us={} p99_us={} commands={submissions} admitted_bytes={admitted_bytes} server_cpu_before={} server_cpu_after={} server_used_memory={}",
        if pipelined { "pipeline32" } else { "serial" },
        micros[14],
        micros[28],
        micros[29],
        cpu(&before),
        cpu(&after),
        field(&memory, "used_memory")
    );
    Ok(())
}

async fn pipelined_payloads(
    reader: &ValkeyReadSession,
    keys: &ValkeyQueueKeys,
    entries: &[ValkeyQueueEntryId],
) -> Result<Vec<Option<String>>, Box<dyn Error>> {
    let mut values = Vec::with_capacity(entries.len());
    for chunk in entries.chunks(32) {
        values.extend(reader.payload_batch(keys, chunk).await?);
    }
    Ok(values)
}

fn field<'a>(info: &'a str, name: &str) -> &'a str {
    info.lines()
        .filter_map(|line| line.split_once(':'))
        .find_map(|(key, value)| (key == name).then_some(value))
        .unwrap_or("unavailable")
}

fn cpu(info: &str) -> String {
    format!(
        "sys:{},user:{}",
        field(info, "used_cpu_sys"),
        field(info, "used_cpu_user")
    )
}
