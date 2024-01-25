use std::collections::HashMap;
use std::env;

use dotenv::dotenv;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

use reqwest;
use std::time::Duration;
use tokio::time::sleep;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message is received - the
    // closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple events can be dispatched
    // simultaneously.
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            // Sending a message can fail, due to a network error, an authentication error, or lack
            // of permissions to post in the channel, so log to stdout when some error happens,
            // with a description of it.
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {why:?}");
            }
        } else if msg.content == "!dice" {
            let random = rand::random::<i32>();
            let res = random % 100 + 1;
            if let Err(why) = msg
                .channel_id
                .say(&ctx.http, format!("{}", res.abs()))
                .await
            {
                println!("Error sending message: {why:?}");
            }
        } else if msg.content == "!pw start" {
            let url = env::var("GCP_API").expect("Expected a token in the environment")
                + "/startinstance";
            let mut map = HashMap::new();
            map.insert("name", "palworld1");
            map.insert("project", "droprealms");
            map.insert("zone", "asia-northeast1-b");
            let client = reqwest::Client::new();
            client.put(url).json(&map).send().await.unwrap();
            if let Err(why) = msg
                .channel_id
                .say(&ctx.http, format!("palworldのサーバーを起動しました。"))
                .await
            {
                println!("Error sending message: {why:?}");
            }
        } else if msg.content == "!pw ip" {
            let url = env::var("GCP_API").expect("Expected a token in the environment") + "/natip ";
            let mut map = HashMap::new();
            map.insert("name", "palworld1");
            map.insert("project", "droprealms");
            map.insert("zone", "asia-northeast1-b");
            let client = reqwest::Client::new();
            let response = client.put(url).json(&map).send().await.unwrap();
            let body = response.text().await.unwrap();
            if let Err(why) = msg
                .channel_id
                .say(&ctx.http, format!("{}", body + ":8211"))
                .await
            {
                println!("Error sending message: {why:?}");
            }
        } else if msg.content == "!pw stop" {
            let url =
                env::var("GCP_API").expect("Expected a token in the environment") + "/stopinstance";
            let mut map = HashMap::new();
            map.insert("name", "palworld1");
            map.insert("project", "droprealms");
            map.insert("zone", "asia-northeast1-b");
            let client = reqwest::Client::new();
            client.put(url).json(&map).send().await.unwrap();
            if let Err(why) = msg
                .channel_id
                .say(&ctx.http, format!("palworldのサーバーを停止しました。"))
                .await
            {
                println!("Error sending message: {why:?}");
            }
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a shard is booted, and
    // a READY payload is sent by Discord. This payload contains data like the current user's guild
    // Ids, current user data, private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
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
