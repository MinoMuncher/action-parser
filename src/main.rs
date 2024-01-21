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


fn sanitize_string(s: &str)->String{
    s.trim_start_matches("\u{feff}").trim_end_matches('\n').trim_end_matches('\r').to_string()
}

use std::{collections::HashMap, error::Error};

use tokio::net::{TcpListener, TcpStream};

async fn handle_client(mut stream: TcpStream) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut reader = BufReader::new(&mut stream);
    let mut filtered_names = String::new();
    reader.read_line(&mut filtered_names).await?;

    let filtered_names: Vec<String> = sanitize_string(&filtered_names)
        .split(',')
        .map(|x| x.to_ascii_lowercase().trim().to_string())
        .filter(|x| x != "")
        .collect();

    let mut num_replays = String::new();
    reader.read_line(&mut num_replays).await?;
    let num_replays: usize = sanitize_string(&num_replays).parse()?;

    let mut replay_strings = Vec::new();

    for _ in 0..num_replays {
        let mut replay = String::new();
        reader.read_line(&mut replay).await?;
        replay_strings.push(replay);
    }

    let mut player_stats: HashMap<String, CumulativePlacementStats> = HashMap::new();

    for replay in replay_strings {
        if let Err(e) = process_replay(&replay, &filtered_names, &mut player_stats).await{
            stream.write_all(format!("{e}\n").as_bytes()).await?;
        }else{
            stream.write_all(format!("success\n").as_bytes()).await?;
        }
    }

    let player_stats: HashMap<_, _> = player_stats
        .into_iter()
        .map(|(username, stats)| (username, PlayerStats::from(&stats)))
        .collect();

    let mut output = serde_json::to_string(&player_stats)?;
    output.push_str("\n");

    stream.write_all(output.as_bytes()).await?;

    stream.shutdown().await?;
    Ok(())
}

#[derive(Debug)]
enum ReplayError{
    Unsupported,
    Unparsable,
    Unmunchable,
    Corrupt,
    Connection
}

impl Error for ReplayError {}

impl std::fmt::Display for ReplayError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self{
            ReplayError::Unsupported => write!(f, "The replay's version is unsupported."),
            ReplayError::Unparsable => write!(f, "The replay was unable to be identified as a valid replay."),
            ReplayError::Unmunchable => write!(f, "The replay's data was unable to be processed into stats."),
            ReplayError::Corrupt => write!(f, "The replay is corrupt, no data was able to be processed."),
            ReplayError::Connection => write!(f, "A connection error occurred with the replay parser."),
        }
    }
}

async fn process_replay(replay: &str, filtered: &[String], player_stats: &mut HashMap<String, CumulativePlacementStats>)->Result<(), ReplayError>{
    let addr: usize = std::env::var("TETRIO_PARSER_PORT").ok().and_then(|s: String| s.parse().ok()).unwrap_or(8080);
    let addr = format!("127.0.0.1:{}",addr).parse().or(Err(ReplayError::Connection))?;

    let socket = TcpSocket::new_v4().or(Err(ReplayError::Connection))?;
    let stream = socket.connect(addr).await.or(Err(ReplayError::Connection))?;
    let mut stream = BufReader::new(stream);
    stream.write_all(sanitize_string(&replay).as_bytes()).await.or(Err(ReplayError::Connection))?;
    stream.write_all("\n".as_bytes()).await.or(Err(ReplayError::Connection))?;
    stream.flush().await.or(Err(ReplayError::Connection))?;

    let mut supported = String::new();
    stream.read_line(&mut supported).await.or(Err(ReplayError::Unparsable))?;
    let supported: bool = sanitize_string(&supported).parse().or(Err(ReplayError::Unparsable))?;

    if !supported{
        return Err(ReplayError::Unsupported)
    }

    let mut names = String::new();
    stream.read_line(&mut names).await.or(Err(ReplayError::Unparsable))?;
    let names: Vec<_> = sanitize_string(&names)
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
    stream.read_line(&mut num_games).await.or(Err(ReplayError::Unparsable))?;
    let num_games: usize = sanitize_string(&num_games).parse().or(Err(ReplayError::Unparsable))?;

    stream.write_all(names.len().to_string().as_bytes()).await.or(Err(ReplayError::Connection))?;
    stream.write_all("\n".as_bytes()).await.or(Err(ReplayError::Connection))?;
    stream.flush().await.or(Err(ReplayError::Connection))?;

    let mut fully_corrupt = true;

    for name in names {
        let mut cumulative_stats = CumulativePlacementStats::default();
        let mut handles = JoinSet::new();

        stream.write_all(name.as_bytes()).await.or(Err(ReplayError::Connection))?; //request stats for [name] from parser
        stream.write_all("\n".as_bytes()).await.or(Err(ReplayError::Connection))?;
        stream.flush().await.or(Err(ReplayError::Connection))?;

        for _ in 0..num_games {
            let mut game = String::new();
            stream.read_line(&mut game).await.or(Err(ReplayError::Unparsable))?; //parse individual placement sequences for each game
            if sanitize_string(&game)=="CORRUPT"{
                continue;
            }
            fully_corrupt = false;
            let placements: Vec<PlacementStats> = serde_json::from_str(&game).or(Err(ReplayError::Unmunchable))?;
            handles
                .spawn_blocking(move || CumulativePlacementStats::from(placements.as_slice()));
        }
        while let Some(handle) = handles.join_next().await {
            let game_stats = handle.or(Err(ReplayError::Unmunchable))?;
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
    if fully_corrupt{
        return Err(ReplayError::Corrupt);
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    let port: usize = std::env::var("ACTION_PARSER_PORT").ok().and_then(|s: String| s.parse().ok()).unwrap_or(8081);
    let port = format!("127.0.0.1:{}",port);
    println!("action parser listening on {}", port);
    let listener = TcpListener::bind(port).await.unwrap();
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
