use std::{fs::{create_dir, read_dir, File}, io::{BufReader, BufWriter, Write}, path::Path, time::SystemTime, collections::HashMap};
use std::sync::atomic::{AtomicBool, Ordering};
use crate::placement_stats::CumulativePlacementStats;

const TIME_TO_LIVE : u64 = 60*60*5; //max secs elapsed to keep file
const CACHE_PATH : &str = ".replayCache";
const MAX_CACHED_FILES: usize = 1000;
const MAX_TRIMMED_FILES: usize = 800;

static TRIMMING_CACHE : AtomicBool = AtomicBool::new(false);

pub fn get_cached_stats(handle: &str) -> Option<HashMap<String, CumulativePlacementStats>>{
    let file_path = Path::new(CACHE_PATH).join(Path::new(handle));
    if !file_path.exists(){
        return None
    }
    let file = File::open(file_path).expect("unable to open cached replay file");
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).ok()
}

pub fn set_cached_stats(handle: &str, players: &HashMap<String, CumulativePlacementStats>){ //supposed to be an endpoint, should we force a consumption?
    let file_path = Path::new(CACHE_PATH).join(Path::new(handle));
    let file = File::create(file_path).expect("unable to open cached replay file");
    let mut writer = BufWriter::new(file);
    writer.write_all(serde_json::to_string(players).expect("unable to serialize stats").as_bytes()).unwrap();
    writer.flush().unwrap();

    let mut files : Vec<_> = read_dir(Path::new(CACHE_PATH)).expect("unable to read replay cache dir").filter_map(|x|x.ok()).filter(|x|x.metadata().unwrap().is_file()).collect();
    if files.len() > MAX_CACHED_FILES && !TRIMMING_CACHE.load(Ordering::SeqCst){
        TRIMMING_CACHE.store(true, Ordering::SeqCst);
        std::thread::spawn(move ||{
            files.sort_by(|x,y|x.metadata().unwrap().modified().unwrap().cmp(&y.metadata().unwrap().modified().unwrap()));
            for i in 0..MAX_TRIMMED_FILES{
                std::fs::remove_file(files[i].path()).expect("unable to remove overflowed file");
            }
            TRIMMING_CACHE.store(false, Ordering::SeqCst);
        });
    }
}

pub fn initialize_cache(){
    let cache_path = Path::new(CACHE_PATH);
    if !cache_path.exists(){
        create_dir(cache_path).expect("unable to create replay cache")
    }

    let now = SystemTime::now();

    for entry in read_dir(cache_path).expect("unable to read replay cache dir"){
        if let Ok(entry) = entry{
            let metadata = entry.metadata().expect("unable to read file metadata");
            if !metadata.is_file(){continue;}
            let created = metadata.created().expect("metadata contains created date");
            if now.duration_since(created).unwrap().as_secs() > TIME_TO_LIVE{
                std::fs::remove_file(entry.path()).expect("unable to remove expired file");
            };
        }
    }
}