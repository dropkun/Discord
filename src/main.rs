use std::collections::HashMap;
use std::env;

use dotenv::dotenv;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use serenity::{async_trait, client};

use reqwest;
use std::time::Duration;
use tokio::time;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message is received - the
    // closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple events can be dispatched
    // simultaneously.
    async fn message(&self, ctx: Context, msg: Message) {
        let mut map = HashMap::new();
        map.insert("name", "palworld1");
        map.insert("project", "droprealms");
        map.insert("zone", "asia-northeast1-b");
        match msg.content.as_str() {
            "!ping" => {
                if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                    println!("Error sending message: {:?}", why);
                }
            }
            "!dice" => {
                let res = rand::random::<i32>() % 100 + 1;
                if let Err(why) = msg.channel_id.say(&ctx.http, res.to_string()).await {
                    println!("Error sending message: {:?}", why);
                }
            }
            "!pw start" => {
                if let Err(why) = start_mc_server(&ctx, &msg, &map).await {
                    println!("Error starting server: {:?}", why);
                }
            }
            "!pw stop" => {
                if let Err(why) = stop_mc_server(&ctx, &msg, &map).await {
                    println!("Error stopping server: {:?}", why);
                }
            }
            _ => {}
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

fn env_var(name: &str) -> String {
    env::var(name).expect("Expected a token in the environment")
}

async fn start_mc_server(
    ctx: &Context,
    msg: &Message,
    map: &HashMap<&str, &str>,
) -> Result<(), reqwest::Error> {
    let url = env_var("GCP_API");
    let client = reqwest::Client::new();

    client
        .post(format!("{}/instance/start", url))
        .json(&map)
        .send()
        .await?;

    if let Err(why) = msg
        .channel_id
        .say(&ctx.http, "palworldのサーバーを起動中...")
        .await
    {
        println!("Error sending message: {:?}", why);
    }

    wait_status("RUNNING".to_string(), &map).await;

    let ip = get_instance_ip(&map).await?;

    if let Err(why) = msg
        .channel_id
        .say(&ctx.http, format!("起動が完了しました。{}", ip + ":8211"))
        .await
    {
        println!("Error sending message: {:?}", why);
    }

    Ok(())
}

async fn stop_mc_server(
    ctx: &Context,
    msg: &Message,
    map: &HashMap<&str, &str>,
) -> Result<(), reqwest::Error> {
    let url = env_var("GCP_API");
    let client = reqwest::Client::new();

    client
        .post(format!("{}/instance/stop", url))
        .json(&map)
        .send()
        .await?;

    if let Err(why) = msg
        .channel_id
        .say(&ctx.http, "palworldのサーバーを停止中...")
        .await
    {
        println!("Error sending message: {:?}", why);
    }

    wait_status("TERMINATED".to_string(), &map).await;

    if let Err(why) = msg.channel_id.say(&ctx.http, "停止が完了しました。").await {
        println!("Error sending message: {:?}", why);
    }

    Ok(())
}

async fn wait_status(status: String, map: &HashMap<&str, &str>) {
    let client = reqwest::Client::new();
    loop {
        let url = env_var("GCP_API");
        let response = client
            .post(format!("{}/instance/status", url))
            .json(&map)
            .send()
            .await
            .unwrap();

        let body = response.text().await.unwrap();
        if body == status {
            break;
        }
        time::sleep(Duration::from_secs(5)).await;
    }
}

async fn get_instance_ip(map: &HashMap<&str, &str>) -> Result<String, reqwest::Error> {
    let url = env_var("GCP_API");
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/instance/ip", url))
        .json(&map)
        .send()
        .await?;
    Ok(response.text().await?)
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot. This will automatically prepend
    // your bot token with "Bot ", which is a requirement by Discord for bot users.
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform exponential backoff until
    // it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
