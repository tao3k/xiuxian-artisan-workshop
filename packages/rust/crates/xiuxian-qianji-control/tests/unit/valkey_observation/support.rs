use std::{
    io,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
    task::JoinHandle,
};

pub struct Server {
    pub url: String,
    pub commands: Arc<AtomicUsize>,
    task: JoinHandle<io::Result<()>>,
}

impl Drop for Server {
    fn drop(&mut self) {
        self.task.abort();
    }
}

pub async fn server(
    mut reply: impl FnMut(&[String]) -> Option<String> + Send + 'static,
) -> io::Result<Server> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let url = format!("redis://{}", listener.local_addr()?);
    let commands = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&commands);
    let task = tokio::spawn(async move {
        let (socket, _) = listener.accept().await?;
        let mut socket = BufReader::new(socket);
        while let Some(command) = read_command(&mut socket).await? {
            let response = if command.first().is_some_and(|cmd| cmd == "CLIENT") {
                Some("+OK\r\n".to_owned())
            } else {
                observed.fetch_add(1, Ordering::SeqCst);
                reply(&command)
            };
            if let Some(response) = response {
                socket.get_mut().write_all(response.as_bytes()).await?;
            } else {
                std::future::pending::<()>().await;
            }
        }
        Ok(())
    });
    Ok(Server {
        url,
        commands,
        task,
    })
}

async fn read_command(
    reader: &mut BufReader<tokio::net::TcpStream>,
) -> io::Result<Option<Vec<String>>> {
    let mut line = String::new();
    if reader.read_line(&mut line).await? == 0 {
        return Ok(None);
    }
    let count = number(&line, '*')?;
    let mut args = Vec::with_capacity(count);
    for _ in 0..count {
        line.clear();
        reader.read_line(&mut line).await?;
        let size = number(&line, '$')?;
        let mut bytes = vec![0; size + 2];
        reader.read_exact(&mut bytes).await?;
        bytes.truncate(size);
        args.push(String::from_utf8(bytes).map_err(io::Error::other)?);
    }
    Ok(Some(args))
}

fn number(line: &str, prefix: char) -> io::Result<usize> {
    line.strip_prefix(prefix)
        .ok_or_else(|| io::Error::other("unexpected RESP prefix"))?
        .trim()
        .parse()
        .map_err(io::Error::other)
}

pub fn bulk(text: &str) -> String {
    format!("${}\r\n{text}\r\n", text.len())
}

pub fn array(values: &[&str]) -> String {
    let mut result = format!("*{}\r\n", values.len());
    for value in values {
        result.push_str(&bulk(value));
    }
    result
}

pub fn index_reply(command: &[String]) -> Option<String> {
    match command[0].as_str() {
        "ZRANGE" => {
            assert_ne!(
                command[3], "-1",
                "observation must not request an unbounded index"
            );
            Some(array(if command[1] == "test:pending" {
                &["run|step"]
            } else {
                &[]
            }))
        }
        "SCAN" => Some(format!("*2\r\n{}{}", bulk("0"), array(&[]))),
        _ => None,
    }
}
