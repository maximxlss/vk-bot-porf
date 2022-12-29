#![feature(never_type)]

use rvk::{APIClient, Params};
use rvk_methods::groups::get_long_poll_server;
use std::future::Future;
use std::time::Duration;
use lazy_static::lazy_static;
use rvk::error::Error;
use rvk_objects::message::Message;


#[macro_export]
macro_rules! param_err {
    () => { rvk::error::Error::from("Failed to get a parameter") }
}


pub async fn poll_for_messages<F, Fut>(api: &APIClient, callback: F) -> rvk::error::Result<!>
where
    F: Fn(Message) -> Fut,
    Fut: Future<Output = ()> + std::marker::Send + 'static,
{
    let mut params = Params::new();
    params.insert("group_id".to_string(), "217990903".to_string());
    let res = get_long_poll_server::<Params>(api, params).await?;
    let server = res.get("server").ok_or(param_err!())?.clone();
    let key = res.get("key").ok_or(param_err!())?.clone();
    let mut ts = res.get("ts").ok_or(param_err!())?.clone();

    loop {
        let body: serde_json::Value = serde_json::from_str(&reqwest::get(format!("{server}?act=a_check&key={key}&ts={ts}&wait=25"))
            .await?
            .text()
            .await?)?;

        ts = body["ts"].as_str().ok_or(param_err!())?.to_string();
        let updates = body["updates"].as_array().ok_or(param_err!())?;
        for update in updates {
            let t = update["type"].as_str().ok_or(param_err!())?;
            if t == "message_new" {
                let msg: Message = serde_json::from_value(update["object"]["message"].clone())?;
                tokio::spawn(callback(msg));
            } else {
                println!("{} - unknown event", t);
            }
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}


lazy_static!{
    static ref REQWEST_CLIENT: reqwest::Client = reqwest::Client::new();
}

pub async fn porfirevich_get(prompt: &str, length: u32) -> rvk::error::Result<String> {
    let mut body = String::with_capacity(prompt.len() + 50);
    body.push_str("{\"prompt\":\"");
    body.push_str(prompt);
    body.push_str("\",\"length\":");
    body.push_str(&length.to_string());
    body.push_str("}");
    body = body.replace("\n", "\\n");
    let res: serde_json::Value = serde_json::from_str(&REQWEST_CLIENT.post("https://pelevin.gpt.dobro.ai/generate/")
        .header("Content-Type", "application/json")
        .body(body)
        .send().await?.text().await?)?;
    let Some(ans) = res["replies"][0].as_str() else {
        println!("weird response from porfirevich: {}", res); return Err(Error::from(""));
    };

    Ok(prompt.to_string() + ans)
}