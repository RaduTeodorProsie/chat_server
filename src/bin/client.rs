use futures::{StreamExt, SinkExt};
use tokio::io::{self, AsyncBufReadExt};
use tokio_tungstenite::tungstenite;


fn read_no_spaces(prompt: &str) -> String {
    use std::io::{self, Write};

    loop {
        print!("{}", prompt);
        io::stdout().flush().unwrap(); // ensure prompt appears

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let trimmed = input.trim();

        if trimmed.is_empty() {
            println!("⚠ Input cannot be empty.");
            continue;
        }

        if trimmed.contains(' ') {
            println!("⚠ No spaces allowed. Please try again.");
            continue;
        }

        return trimmed.to_string();
    }
}
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let nickname = read_no_spaces("===== What's your nickname for today? (No spaces) =====\n");
    let room = read_no_spaces("===== What room do you want to join? (No spaces) =====\n");

    let nickname = nickname.trim_matches(|c| c == '\r' || c == '\n').to_string();
    let room = room.trim_matches(|c| c == '\r' || c == '\n').to_string();
    println!("Nickname: {}, Room: {}", nickname, room);

    let ip = "localhost:8080";
    let link = format!("ws://{}/login/{}/{}", ip, room, nickname);

    println!("Connecting...");
    let (ws_stream, _) = tokio_tungstenite::connect_async(link).await.expect("Failed to connect to server");

    println!("Connected!");
    let (mut write, mut read) = ws_stream.split();

    tokio::spawn(async move {
        while let Some(Ok(msg)) = read.next().await {
            println!("{}", msg);
        }
    });

    let stdin = io::BufReader::new(io::stdin());
    let mut lines = stdin.lines();

    while let Ok(Some(line)) = lines.next_line().await {
        write.send(tungstenite::Message::Text(line.into())).await?;
    }

    Ok(())
}