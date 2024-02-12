mod attack;
mod board_analyzer;
mod placement_stats;
mod player_stats;
mod replay_response;
mod solver;
mod cache;
mod io;

use placement_stats::CumulativePlacementStats;
use player_stats::PlayerStats;
use replay_response::PlacementStats;
use std::sync::Arc;
use std::{collections::HashMap, error::Error};
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpSocket, TcpStream};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    task::JoinSet,
};
use cache::{initialize_cache, get_cached_stats, set_cached_stats};
use io::{download_replay, io_auth};

///removes wrapper characters around tcp streams
fn sanitize_string(s: &str) -> String {
    s.trim_start_matches('\u{feff}')
        .trim_end_matches('\n')
        .trim_end_matches('\r')
        .to_string()
}

///handling the client
async fn handle_client(stream: TcpStream, opts: Arc<RunOpts>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut stream = BufReader::new(stream);
    let mut filtered_names = String::new();
    stream.read_line(&mut filtered_names).await?;
    //list of names to request from replay, empty list means take all available names from replay

    let filtered_names: Vec<String> = sanitize_string(&filtered_names)
        .split(',')
        .map(|x| x.to_ascii_lowercase().trim().to_string())
        .filter(|x| !x.is_empty())//sanity check to remove double comma case
        .collect();
    //map names to lowercase, so that name searching is case insensitive

    let mut player_stats: HashMap<String, CumulativePlacementStats> = HashMap::new();
    //we keep a map of players' cumulative placemenet stats, then transform to advanced stats after


    let mut num_replay_ids = String::new();
    stream.read_line(&mut num_replay_ids).await?;
    let num_replay_ids: usize = sanitize_string(&num_replay_ids).parse()?;
    //get number of replays to be loaded

    for _ in 0..num_replay_ids {
        let mut replay_id = String::new();
        stream.read_line(&mut replay_id).await?;

        let cached_stats = if opts.caching_enabled{
            let cached_stats = get_cached_stats(&replay_id);
            let cached_stats = match cached_stats{
                None=>{Some(HashMap::new())},
                Some(stats)=>{
                    if (filtered_names.len()==0 && stats.len() == 2) || filtered_names.iter().all(|name| stats.keys().any(|cached_name|&cached_name.to_lowercase()==name)){
                        for (name, cumulative_stats) in stats{
                            if filtered_names.len() > 0 && !filtered_names.contains(&name.to_lowercase()){continue;}
                            match player_stats.entry(name) {
                                std::collections::hash_map::Entry::Occupied(mut entry) => {
                                    entry.get_mut().absorb(cumulative_stats);
                                }
                                std::collections::hash_map::Entry::Vacant(entry) => {
                                    entry.insert(cumulative_stats);
                                }
                            }
                        }
                        write_line(&mut stream, "success").await?;
                        continue;
                    }
                    Some(stats)
                }
            };
            cached_stats
        }else{
            None
        };

        let replay = download_replay(&replay_id, &opts.token).await;
        let replay = match replay{
            Ok(replay) => replay,
            Err(e) => {
                eprintln!("ERROR DOWNLOADING REPLAY: {}", e);
                write_line(&mut stream, "error downloading replay").await?;
                continue;
            },
        };
    
        if let Err(e) = process_replay(&replay, &filtered_names, &mut player_stats, &replay_id, cached_stats).await {
            write_line(&mut stream, &format!("{e}")).await?;
        } else {
            write_line(&mut stream, "success").await?;
        }
    }

    let mut num_replays = String::new();
    stream.read_line(&mut num_replays).await?;
    let num_replays: usize = sanitize_string(&num_replays).parse()?;
    //get number of replays to be loaded

    for _ in 0..num_replays {
        let mut hash = String::new();
        stream.read_line(&mut hash).await?;


        let cached_stats = if opts.caching_enabled{
            let cached_stats = get_cached_stats(&hash);
            let cached_stats = match cached_stats{
                None=>{
                    Some(HashMap::new())
                },
                Some(stats)=>{
                    if (filtered_names.len()==0 && stats.len() == 2) || filtered_names.iter().all(|name| stats.keys().any(|cached_name|&cached_name.to_lowercase()==name)){
                        for (name, cumulative_stats) in stats{
                            if filtered_names.len() > 0 && !filtered_names.contains(&name.to_lowercase()){continue;}
                            match player_stats.entry(name) {
                                std::collections::hash_map::Entry::Occupied(mut entry) => {
                                    entry.get_mut().absorb(cumulative_stats);
                                }
                                std::collections::hash_map::Entry::Vacant(entry) => {
                                    entry.insert(cumulative_stats);
                                }
                            }
                        }
                        write_line(&mut stream, "true").await?;
                        continue;
                    }
                    Some(stats)
                }
            };
            write_line(&mut stream, "false").await?;
            cached_stats
        }else{
            write_line(&mut stream, "false").await?;
            None
        };

        let mut replay = String::new();
        stream.read_line(&mut replay).await?;

        if let Err(e) = process_replay(&replay, &filtered_names, &mut player_stats, &hash, cached_stats).await {
            write_line(&mut stream, &format!("{e}")).await?;
        } else {
            write_line(&mut stream, "success").await?;
        }
    }

    let player_stats: HashMap<_, _> = player_stats
        .into_iter()
        .map(|(username, stats)| (username, PlayerStats::from(&stats)))
        .collect();
    //transform player stats

    let output = serde_json::to_string(&player_stats)?;
    write_line(&mut stream, &output).await?;
    //write player stats
    stream.shutdown().await?;
    Ok(())
}

#[derive(Debug)]
enum ReplayError {
    Unsupported,
    Unparsable,
    Unmunchable,
    Corrupt,
    Connection,
}

impl Error for ReplayError {}

impl std::fmt::Display for ReplayError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ReplayError::Unsupported => write!(f, "The replay's version is unsupported."),
            ReplayError::Unparsable => write!(
                f,
                "The replay was unable to be identified as a valid replay."
            ),
            ReplayError::Unmunchable => write!(
                f,
                "The replay's data was unable to be processed into stats."
            ),
            ReplayError::Corrupt => write!(
                f,
                "The replay is corrupt, no data was able to be processed."
            ),
            ReplayError::Connection => {
                write!(f, "A connection error occurred with the replay parser.")
            }
        }
    }
}

async fn write_line(stream: &mut BufReader<TcpStream>, line: &str)-> Result<(), ReplayError>{
    let line_bytes = line.as_bytes();
    stream.write_all(line_bytes).await.or(Err(ReplayError::Connection))?;
    stream.write_u8('\n' as u8).await.or(Err(ReplayError::Connection))?;
    stream.flush().await.or(Err(ReplayError::Connection))?;
    Ok(())
}

async fn process_replay(
    replay: &str,
    filtered: &[String],
    player_stats: &mut HashMap<String, CumulativePlacementStats>,
    cached_handle: &str,
    mut cached_stats: Option<HashMap<String, CumulativePlacementStats>> //mutable cache to save later
) -> Result<(), ReplayError> {

    let mut cached_stats_updated = false;

    let addr: usize = std::env::var("TETRIO_PARSER_PORT")
        .ok()
        .and_then(|s: String| s.parse().ok())
        .unwrap_or(8080);
    //address of the modded csdotnet replay parser tcp connection, default 8080

    let addr = format!("127.0.0.1:{}", addr)
        .parse()
        .or(Err(ReplayError::Connection))?;

    let socket = TcpSocket::new_v4().or(Err(ReplayError::Connection))?;
    let stream = socket
        .connect(addr)
        .await
        .or(Err(ReplayError::Connection))?;
    let mut stream = BufReader::new(stream);

    write_line(&mut stream, &sanitize_string(replay)).await?;
    //write replay string

    let mut supported = String::new();
    stream
        .read_line(&mut supported)
        .await
        .or(Err(ReplayError::Unparsable))?;
    let supported: bool = sanitize_string(&supported)
        .parse()
        .or(Err(ReplayError::Unparsable))?;

    if !supported {
        return Err(ReplayError::Unsupported);
    }
    //ask parser if version is supported or not

    let mut names = String::new();
    stream
        .read_line(&mut names)
        .await
        .or(Err(ReplayError::Unparsable))?;
    let names: Vec<_> = sanitize_string(&names)
        .split(' ')
        .map(|s| s.to_string())
        .collect();
    //get names in replay

    let names = if filtered.is_empty() {
        names
    } else {
        names
            .into_iter()
            .filter(|x| filtered.contains(&x.to_lowercase()))
            .collect()
    };
    //if filtered name list is empty, don't modify. else filter with case insensitivity

    let mut num_games = String::new();
    stream
        .read_line(&mut num_games)
        .await
        .or(Err(ReplayError::Unparsable))?;
    let num_games: usize = sanitize_string(&num_games)
        .parse()
        .or(Err(ReplayError::Unparsable))?;
    //get number of games of replay

    write_line(&mut stream, &names.len().to_string()).await?;
    //write number of names to get stats for

    let mut fully_corrupt = true;

    for name in names {
        let mut cumulative_stats = CumulativePlacementStats::default();
        let mut handles = JoinSet::new();
        //joinset to process stat transformation multithreadedly

        write_line(&mut stream, &name).await?;

        for _ in 0..num_games {
            let mut game = String::new();
            stream
                .read_line(&mut game)
                .await
                .or(Err(ReplayError::Unparsable))?; //parse individual placement sequences for each game
            if sanitize_string(&game) == "CORRUPT" {
                continue;
            }
            fully_corrupt = false;
            let placements: Vec<PlacementStats> =
                serde_json::from_str(&game).or(Err(ReplayError::Unmunchable))?; //something went wrong in the response loop, error should never happen
            handles.spawn_blocking(move || CumulativePlacementStats::from(placements.as_slice()));
            //create handle to parse stats, this from operation is heavy
        }
        while let Some(handle) = handles.join_next().await {
            let game_stats = handle.or(Err(ReplayError::Unmunchable))?;
            cumulative_stats.absorb(game_stats); //join all handles and their respective stats
        }
         
        if let Some(mut stats) = cached_stats{
            if !stats.contains_key(&name){
                stats.insert(name.clone(), cumulative_stats.clone()); //how can i avoid this clone?
                cached_stats_updated = true;
            }
            cached_stats = Some(stats);
        }

        match player_stats.entry(name) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                entry.get_mut().absorb(cumulative_stats);
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(cumulative_stats);
            }
        }
        //merge stats for respective player
    }
    if let Some(cached_stats) = cached_stats{
        if cached_stats_updated{
            set_cached_stats(cached_handle, &cached_stats);
        }    
    }

    if fully_corrupt {
        return Err(ReplayError::Corrupt);
    } //no data received? return corrupt

    Ok(())
}

struct RunOpts{
    caching_enabled: bool,
    token: String
}

#[tokio::main]
async fn main() {
    let caching_enabled : bool = std::env::var("ENABLE_CACHE").ok()
    .and_then(|s: String| s.parse().ok())
    .unwrap_or(true);
    initialize_cache();

    let token = io_auth().await;

    let opts = RunOpts{
        token,
        caching_enabled
    };

    let shared_opts = Arc::new(opts);
    
    let port: usize = std::env::var("ACTION_PARSER_PORT")
        .ok()
        .and_then(|s: String| s.parse().ok())
        .unwrap_or(8081);
    //listen on port, default 8081

    let port = format!("127.0.0.1:{}", port);
    println!("action parser listening on {}", port);
    let listener = TcpListener::bind(port).await.unwrap();
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let cloned_opts = Arc::clone(&shared_opts);
                tokio::spawn(async move {
                    if let Err(e) = handle_client(stream, cloned_opts).await {
                        eprintln!("error handling client! {}", e);
                    };
                });
            }
            Err(e) => eprintln!("Error accepting connection: {:?}", e),
        }
    }
}
