use std::{fs, path::{Path, PathBuf}, net::TcpStream, io::{BufReader, BufRead, Write, BufWriter, Read}, thread, fs::File, collections::HashMap, time::Instant};
use action_parser::{placement_stats::CumulativePlacementStats, replay_response::PlacementStats, player_stats::PlayerStats};
use notify::{Config as WatcherConfig, RecommendedWatcher, RecursiveMode, Watcher};


fn main() {
    let path = std::env::args()
        .nth(1).unwrap_or("replays".to_string());
    let path = Path::new(&path); //path to be watching for new replays, recursive watch
    assert!(path.is_dir(), "input path is a directory");
    println!("watching {:?}", path.canonicalize().expect("able to canonize path")); 

    let out_path = std::env::args()
    .nth(2).unwrap_or("output".to_string());
    let out_path = Path::new(&out_path);
    assert!(path.is_dir(), "output path is a directory");
    println!("outputting at {:?}", out_path.canonicalize().expect("able to canonize path")); //path to be writing to for new replays

    let mut file_data: HashMap<PathBuf, Vec<(String, CumulativePlacementStats)>> = HashMap::new();
    
    for entry in std::fs::read_dir(path).expect("able to read output path"){
        let path = entry.expect("file entry error").path();
        if path.extension().and_then(|ext|ext.to_str()).is_some_and(|ext|ext == "ttrm" || ext == "ttr"){
            parse_file(path, &mut file_data);
        }
    }
    output_data(out_path, &file_data);

    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher = RecommendedWatcher::new(
        tx,
        WatcherConfig::default()
    ).expect("error creating file path watcher");

    watcher.watch(path, RecursiveMode::Recursive).unwrap();

    while let Ok(Ok(e)) = rx.recv(){ // ??? bullshit lazy handling
        let paths : Vec<_> = e.paths.into_iter().filter(|path|{
            path.extension().and_then(|ext|ext.to_str()).is_some_and(|ext|ext == "ttrm" || ext == "ttr")
        }).collect();

        if paths.is_empty() {continue;}

        for path in paths{
            println!("processing path {:?}", path);
            if path.exists(){
                parse_file(path, &mut file_data);
            }else{
                file_data.remove(&path);
                println!("{:?}", file_data.keys());
                println!("removing data from {:?}", path);
            }
        }

        output_data(out_path, &file_data);
    }
    
}

fn parse_file(path: PathBuf, file_data: &mut HashMap<PathBuf, Vec<(String, CumulativePlacementStats)>>){
    let file = File::open(&path).expect("unable to access files");
    let mut reader = BufReader::new(file);
    let mut replay = String::new();
    reader.read_to_string(&mut replay).expect("unable to read files");
    let instant = Instant::now();
    match process_replay(&replay){
        Ok(players) => {
            println!("successfully parsed file at {:?} in {}ms", path, instant.elapsed().as_millis());
            file_data.insert(path, players);
        },
        Err(e) => {
            println!("error parsing file at {:?}: {}", path, e);
        },
    }
}

fn output_data(path: &Path, file_data: &HashMap<PathBuf, Vec<(String, CumulativePlacementStats)>>){
    let mut player_stats: HashMap<String, CumulativePlacementStats> = HashMap::new();
    for players in file_data.values(){
        for (name, cumulative_stats) in players{
            match player_stats.entry(name.to_string()) {
                std::collections::hash_map::Entry::Occupied(mut entry) => {
                    entry.get_mut().absorb_ref(cumulative_stats);
                }
                std::collections::hash_map::Entry::Vacant(entry) => {
                    entry.insert(cumulative_stats.clone());
                }
            }
        }
    }

    fs::write(path.join("raw_output.json"), serde_json::to_string(&player_stats).expect("error serializing")).expect("error writing");

    let player_stats: HashMap<_, _> = player_stats
    .into_iter()
    .map(|(username, stats)| (username, PlayerStats::from(&stats)))
    .collect();

    fs::write(path.join("output.json"), serde_json::to_string(&player_stats).expect("error serializing")).expect("error writing");

    println!("WROTE DATA");
}


#[derive(Debug)]
enum ReplayError{
    Unsupported,
    Unparsable,
    Unmunchable,
    Corrupt,
    Connection
}

impl std::error::Error for ReplayError {}

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


fn sanitize_string(s: &str)->String{
    s.trim_start_matches('\u{feff}').trim_end_matches('\n').trim_end_matches('\r').to_string()
}


fn process_replay(replay: &str)->Result<Vec<(String, CumulativePlacementStats)>, ReplayError>{
    let addr: usize = std::env::var("TETRIO_PARSER_PORT").ok().and_then(|s: String| s.parse().ok()).unwrap_or(8080);
    let addr = format!("127.0.0.1:{}",addr);

    let stream = TcpStream::connect(addr).or(Err(ReplayError::Connection))?;
    let mut reader = BufReader::new(stream.try_clone().or(Err(ReplayError::Connection))?);
    let mut writer = BufWriter::new(stream);
    writer.write_all(sanitize_string(replay).as_bytes()).or(Err(ReplayError::Connection))?;
    writer.write_all("\n".as_bytes()).or(Err(ReplayError::Connection))?;
    writer.flush().or(Err(ReplayError::Connection))?;

    let mut supported = String::new();
    reader.read_line(&mut supported).or(Err(ReplayError::Unparsable))?;
    let supported: bool = sanitize_string(&supported).parse().or(Err(ReplayError::Unparsable))?;

    if !supported{
        return Err(ReplayError::Unsupported)
    }

    let mut names = String::new();
    reader.read_line(&mut names).or(Err(ReplayError::Unparsable))?;
    let names: Vec<_> = sanitize_string(&names)
        .split(' ')
        .map(|s| s.to_string())
        .collect();

    let mut num_games = String::new();
    reader.read_line(&mut num_games).or(Err(ReplayError::Unparsable))?;
    let num_games: usize = sanitize_string(&num_games).parse().or(Err(ReplayError::Unparsable))?;

    writer.write_all(names.len().to_string().as_bytes()).or(Err(ReplayError::Connection))?;
    writer.write_all("\n".as_bytes()).or(Err(ReplayError::Connection))?;
    writer.flush().or(Err(ReplayError::Connection))?;

    let mut fully_corrupt = true;

    let mut stats = Vec::new();

    for name in names {
        let mut cumulative_stats = CumulativePlacementStats::default();
        let mut handles = Vec::new();

        writer.write_all(name.as_bytes()).or(Err(ReplayError::Connection))?; //request stats for [name] from parser
        writer.write_all("\n".as_bytes()).or(Err(ReplayError::Connection))?;
        writer.flush().or(Err(ReplayError::Connection))?;

        for _ in 0..num_games {
            let mut game = String::new();
            reader.read_line(&mut game).or(Err(ReplayError::Unparsable))?; //parse individual placement sequences for each game
            if sanitize_string(&game)=="CORRUPT"{
                continue;
            }
            fully_corrupt = false;
            let placements: Vec<PlacementStats> = serde_json::from_str(&game).or(Err(ReplayError::Unmunchable))?;

            handles.push(thread::spawn(move || CumulativePlacementStats::from(placements.as_slice())));
        }
         
        while let Some(handle) = handles.pop(){
            let game_stats = handle.join().or(Err(ReplayError::Unmunchable))?;
            cumulative_stats.absorb(game_stats);
        }
        stats.push((name, cumulative_stats))
    }
    if fully_corrupt{
        return Err(ReplayError::Corrupt);
    }
    Ok(stats)
}