mod attack;
mod board_analyzer;
mod placement_stats;
mod player_stats;
mod replay_response;
mod solver;

use placement_stats::CumulativePlacementStats;
use player_stats::PlayerStats;
use replay_response::PlacementStats;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpSocket;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    task::JoinSet,
};

use std::{collections::HashMap, error::Error};

use tokio::net::{TcpListener, TcpStream};

async fn handle_client(mut stream: TcpStream) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut reader = BufReader::new(&mut stream);
    let mut filtered_names = String::new();
    reader.read_line(&mut filtered_names).await?;

    let filtered_names: Vec<String> = filtered_names
        .trim()
        .to_string()
        .split(',')
        .map(|x| x.to_ascii_lowercase().trim().to_string())
        .filter(|x| x != "")
        .collect();

    let mut num_replays = String::new();
    reader.read_line(&mut num_replays).await?;
    let num_replays: usize = num_replays.trim().parse()?;

    let mut replay_strings = Vec::new();

    for _ in 0..num_replays {
        let mut replay = String::new();
        reader.read_line(&mut replay).await?;
        replay_strings.push(replay);
    }

    match process_replays(&replay_strings, &filtered_names).await {
        Ok(output) => {
            stream.write_all(output.as_bytes()).await?;
            stream.flush().await?;
        }
        Err(e) => {
            stream.write_all("error processing replay".as_bytes()).await?;
            stream.flush().await?;
            eprintln!("ERROR PROCESSING REPLAY {e}");
        }
    };

    Ok(())
}

async fn process_replays(
    replays: &[String],
    filtered: &[String],
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let mut player_stats: HashMap<String, CumulativePlacementStats> = HashMap::new();

    for replay in replays {
        let addr = "127.0.0.1:8080".parse()?;

        let socket = TcpSocket::new_v4()?;
        let stream = socket.connect(addr).await?;
        let mut stream = BufReader::new(stream);
        stream.write_all(replay.trim().as_bytes()).await?;
        stream.write_all("\n".as_bytes()).await?;
        stream.flush().await?;

        let mut supported = String::new();
        stream.read_line(&mut supported).await?;
        let supported: bool = supported.trim_start_matches("\u{feff}").trim_end_matches('\n').parse()?;

        if !supported{
            return Ok("unsupported replay version".to_string());
        }

        let mut names = String::new();
        stream.read_line(&mut names).await?;
        let names: Vec<_> = names
            .trim_start_matches("\u{feff}")
            .trim_end_matches('\n')
            .split(' ')
            .map(|s| s.to_string())
            .collect();

        let names = if filtered.len() == 0 {
            names
        } else {
            names
                .into_iter()
                .filter(|x| filtered.contains(&x.to_lowercase()))
                .collect()
        };

        let mut num_games = String::new();
        stream.read_line(&mut num_games).await?;
        let num_games: usize = num_games.trim_start_matches("\u{feff}").trim_end_matches('\n').parse()?;

        stream.write_all(names.len().to_string().as_bytes()).await?;
        stream.write_all("\n".as_bytes()).await?;
        stream.flush().await?;

        for name in names {
            let mut cumulative_stats = CumulativePlacementStats::default();
            let mut handles = JoinSet::new();

            stream.write_all(name.as_bytes()).await?; //request stats for [name] from parser
            stream.write_all("\n".as_bytes()).await?;
            stream.flush().await?;

            for _ in 0..num_games {
                let mut game = String::new();
                stream.read_line(&mut game).await?; //parse individual placement sequences for each game
                let placements: Vec<PlacementStats> = serde_json::from_str(&game)?;
                handles
                    .spawn_blocking(move || CumulativePlacementStats::from(placements.as_slice()));
            }
            while let Some(handle) = handles.join_next().await {
                let game_stats = handle?;
                cumulative_stats.absorb(game_stats);
            }

            match player_stats.entry(name.to_string()) {
                std::collections::hash_map::Entry::Occupied(mut entry) => {
                    entry.get_mut().absorb(cumulative_stats);
                }
                std::collections::hash_map::Entry::Vacant(entry) => {
                    entry.insert(cumulative_stats);
                }
            }
        }
    }

    let player_stats: HashMap<_, _> = player_stats
        .into_iter()
        .map(|(username, stats)| (username, PlayerStats::from(&stats)))
        .collect();

    let output = serde_json::to_string(&player_stats)?;
    Ok(output)
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8081").await.unwrap();
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                tokio::spawn(async move {
                    if let Err(e) = handle_client(stream).await {
                        eprintln!("error handling client! {}", e);
                    };
                });
            }
            Err(e) => eprintln!("Error accepting connection: {:?}", e),
        }
    }
}
