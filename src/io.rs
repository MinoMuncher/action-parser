use std::env::var;

use reqwest::header;
use serde::Serialize;

#[derive(Serialize)]
struct AuthBody{
    username : String,
    password : String
}

pub async fn io_auth()->String{
    let auth_body = AuthBody{
        username : var("TETRIO_USERNAME").unwrap(),
        password : var("TETRIO_PASSWORD").unwrap()
    };


    let client = reqwest::Client::new();
    let res = client.post("https://tetr.io/api/users/authenticate")
    .header(header::CONTENT_TYPE, "application/json")
    .header(header::ACCEPT, "application/json")
    .json(&auth_body)
    .send().await;

    let res : serde_json::Value = res.unwrap().json().await.unwrap();
    res.get("token").unwrap().as_str().unwrap().to_owned()
}
#[derive(Debug)]
pub enum DownloadError {
    Unsuccessful,
    Corrupted,
    Request(reqwest::Error)
}

impl std::error::Error for DownloadError {}

impl std::fmt::Display for DownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DownloadError::Corrupted=>f.write_str("replay downloaded but corrupted"),
            DownloadError::Unsuccessful=>f.write_str("replay unable to be downloaded"),
            DownloadError::Request(e)=>f.write_fmt(format_args!("replay download{e}"))
        }
    }
}


pub async fn download_replay(id: &str, token: &str)->Result<String, DownloadError>{
    let client = reqwest::Client::new();
    let res = match client.get(&format!("https://tetr.io/api/games/{id}"))
    .header(header::ACCEPT, "application/json")
    .header(header::AUTHORIZATION, token)
    .send().await{Ok(res)=>res, Err(e)=>{return Err(DownloadError::Request(e))}};
    let res = match res.error_for_status(){
        Err(e)=>{
            return Err(DownloadError::Request(e))
        },
        Ok(res)=>res
    };
    let response : serde_json::Value = res.json().await.or(Err(DownloadError::Corrupted))?;

    let success = match response.get("success"){
        Some(s)=>s,
        None=>{
            return Err(DownloadError::Corrupted);
        }
    };
    if success.is_boolean(){
        if success != &serde_json::Value::Bool(true){
            return Err(DownloadError::Unsuccessful)
        }
    }else{
        return Err(DownloadError::Corrupted)
    };
    let game = match response.get("game"){
        Some(s)=>s,
        None=>{
            return Err(DownloadError::Corrupted)
        }
    };
    if game.is_object(){
        return Ok(game.to_string())
    }else{
        return Err(DownloadError::Corrupted)
    };
}