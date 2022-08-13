use std::env;

use function_name::named;
use lazy_static::lazy_static;
use regex::Regex;
use serenity::async_trait;
use serenity::http::Http;
use serenity::model::id::ChannelId;
use serenity::prelude::*;
use serenity::model::prelude::Message;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{StandardFramework, CommandResult};

lazy_static!{
    // Match messages of the form 
    // 'https://discord.com/channels/{numeric}/{numeric}/{numeric}' and
    // 'http://discord.com/channels/{numeric}/{numeric}/{numeric}'
    static ref URL_RE: Regex = Regex::new(r"^https?://discord.com/channels/([0-9]+)/([0-9]+)/([0-9]+)$").unwrap();
}

const COMMAND_PREFIX: &str = "~";

#[group]
#[commands(pin, unpin)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {}

#[derive(Debug)]
#[derive(PartialEq)]
enum UrlParseResults {
    Success(u64, u64, u64),
    Invalid
}

#[tokio::main]
async fn main() {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix(COMMAND_PREFIX)) // set the bot's prefix to "-"
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("ENV::DISCORD_TOKEN");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

/// Splits a url of the format http[s?]://discord.com/channels/{sv}/{ch}/{id} into {sv}, {ch}, {id}.
fn validate_url(message: &str) -> UrlParseResults {
    match URL_RE.captures(message) {
        None => {
            UrlParseResults::Invalid
        }, Some(captures) => {
            UrlParseResults::Success(
                captures.get(1).unwrap().as_str().parse().unwrap(),
                captures.get(2).unwrap().as_str().parse().unwrap(),
                captures.get(3).unwrap().as_str().parse().unwrap())
        }
    }
}

async fn send_message_print_err(channel_id: ChannelId, http: impl AsRef<Http>, content: impl std::fmt::Display) {
    if let Err(e) = channel_id.say(http, content).await {
        println!("{}", e);
    }
}

async fn process_inputs(strip_prefix: &str, ctx: &Context, msg: &Message) -> Option<Message> {
    match msg.content.strip_prefix(strip_prefix) {
        None => {
            send_message_print_err(msg.channel_id, &ctx.http, "Input required: message URL").await;
            None
        }, Some(url) => {
            match validate_url(url) {
                UrlParseResults::Invalid => {
                    send_message_print_err(msg.channel_id, &ctx.http, "Invalid message URL").await;
                    None
                }, UrlParseResults::Success(_, ch, mg) => {
                    match ctx.http.get_message(ch, mg).await {
                        Err(_) => {
                            send_message_print_err(msg.channel_id, &ctx.http, "Couldn't get message").await;
                            None
                        }, Ok(message) => {
                            Some(message)
                        }
                    }
                }
            }
        }
    }
}

#[named]
#[command]
async fn pin(ctx: &Context, msg: &Message) -> CommandResult {
    //println!("{:?}", msg);
    let strip_prefix = COMMAND_PREFIX.to_owned() + function_name!() + " ";

    if let Some(message) = process_inputs(&strip_prefix, ctx, msg).await {
        if message.pin(&ctx.http).await.is_err() {
            send_message_print_err(msg.channel_id, &ctx.http, "Insufficient permissions").await;
        }
    }
    Ok(())
}

#[named]
#[command]
async fn unpin(ctx: &Context, msg: &Message) -> CommandResult {
    //println!("{:?}", msg);
    let strip_prefix = COMMAND_PREFIX.to_owned() + function_name!() + " ";

    if let Some(message) = process_inputs(&strip_prefix, ctx, msg).await {
        if message.unpin(&ctx.http).await.is_err() {
            send_message_print_err(msg.channel_id, &ctx.http, "Insufficient permissions").await;
        }
    }
    Ok(())
}

#[cfg(test)]
mod url_validation_test {
    use super::*;
    use super::UrlParseResults::*;

    #[test]
    fn good_https() {
        assert_eq!(
            validate_url("https://discord.com/channels/432708847304704010/432708847304704013/1008041568613191813"), 
            Success(432708847304704010, 432708847304704013, 1008041568613191813)
        );
    }

    #[test]
    fn good_http() {
        assert_eq!(
            validate_url("http://discord.com/channels/432708847304704010/432708847304704013/1008041568613191813"), 
            Success(432708847304704010, 432708847304704013, 1008041568613191813)
        );
    }

    #[test]
    fn prefix() {
        assert_eq!(
            validate_url("prefixhttps://discord.com/channels/432708847304704010/432708847304704013/1008041568613191813"),
            Invalid
        );
    }
    
    #[test]
    fn suffix() {
        assert_eq!(
            validate_url("https://discord.com/channels/432708847304704010/432708847304704013/1008041568613191813suffix"),
            Invalid
        );
    }

    #[test]
    fn protocol() {
        assert_eq!(
            validate_url("ftp://discord.com/channels/432708847304704010/432708847304704013/1008041568613191813"),
            Invalid
        );
    }

    #[test]
    fn endpoint() {
        assert_eq!(
            validate_url("https://discord.com/nonsense/432708847304704010/432708847304704013/1008041568613191813"),
            Invalid
        );
    }
    
    #[test]
    fn nonsense() {
        assert_eq!(validate_url("not a url at all"), Invalid);
    }

    #[test]
    fn non_numeric() {
        assert_eq!(validate_url("https://discord.com/channels/039/not/numeric"), Invalid);
    }

    #[test]
    fn not_enough_segments() {
        assert_eq!(validate_url("https://discord.com/channels/432708847304704010/43"), Invalid);
    }

    #[test]
    fn empty() {
        assert_eq!(validate_url("https://discord.com/channels///"), Invalid);
    }
}