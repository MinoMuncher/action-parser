mod attack;
mod board_analyzer;
mod placement_stats;
mod player_stats;
mod replay_response;
mod solver;

use async_std::io::prelude::BufReadExt;
use async_std::io::{BufReader, WriteExt};
use async_std::stream::StreamExt;
use placement_stats::CumulativePlacementStats;
use player_stats::PlayerStats;
use replay_response::ReplayResponse;

use std::collections::HashMap;
use std::error::Error;

use async_std::net::{TcpListener, TcpStream};

async fn handle_client(mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut reader = BufReader::new(&stream);
    let mut filtered_names = String::new();
    reader.read_line(&mut filtered_names).await?;

    let filtered_names: Vec<String> = filtered_names
    .trim()
    .to_string()
    .split(',')
    .map(|x| x.to_ascii_lowercase().trim().to_string())
    .filter(|x|x!="")
    .collect();


    let mut num_replays = String::new();
    reader.read_line(&mut num_replays).await?;
    let num_replays = num_replays.trim();

    let num_replays: usize = num_replays.parse()?;

    let mut replay_strings = Vec::new();

    for _ in 0..num_replays{
        let mut replay = String::new();
        reader.read_line(&mut replay).await?;
        replay_strings.push(replay);
    }



    match process_replays(replay_strings, filtered_names).await {
        Ok(output) => {
            stream.write_all(output.as_bytes()).await?;
            stream.flush().await?;
        },
        Err(e) => {
            eprintln!("ERROR PROCESSING REPLAY {e}");
        }
    };

    Ok(())
}

async fn process_replays(
    replays: Vec<String>,
    filtered: Vec<String>,
) -> Result<String, Box<dyn Error>> {
    let mut placement_stats = HashMap::new();
    for replay in replays {

        let res: ReplayResponse = serde_json::from_str(&replay)?;
        
        for (username, mut player_placements) in res.player_logs {
            if filtered.len()!=0 && !filtered.contains(&username.to_ascii_lowercase()) {
                continue;
            }
            match placement_stats.entry(username) {
                std::collections::hash_map::Entry::Vacant(entry) => {
                    entry.insert(player_placements);
                }
                std::collections::hash_map::Entry::Occupied(mut entry) => {
                    entry.get_mut().append(&mut player_placements);
                }
            }
        }
    }



    let player_stats : HashMap<_,_>= placement_stats.into_iter().map(|(username, placements)| {
        let stats = CumulativePlacementStats::from(&placements);
        let stats = PlayerStats::from(&stats);
        (username, stats)
    }).collect();



    let output = serde_json::to_string(&player_stats)?;
    Ok(output)
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8081").await.unwrap();
    let mut incoming = listener.incoming();
    while let Some(stream) = incoming.next().await {
        if let Ok(stream) = stream {
            if let Err(e) = handle_client(stream).await {
                println!("handling client err {e}");
            }
        } else {
            println!("STREAM C ERR > V BAD")
        }
    }
}
