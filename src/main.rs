use std::env;

use function_name::named;
use lazy_static::lazy_static;
use regex::Regex;
use serenity::async_trait;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{CommandResult, StandardFramework};
use serenity::http::Http;
use serenity::model::id::ChannelId;
use serenity::model::prelude::Message;
use serenity::prelude::*;

lazy_static! {
    // Match messages of the form
    // 'https://discord.com/channels/{numeric}/{numeric}/{numeric}' and
    // 'http://discord.com/channels/{numeric}/{numeric}/{numeric}'
    static ref URL_RE: Regex = Regex::new(r"^https?://discord.com/channels/([0-9]+)/([0-9]+)/([0-9]+)$").unwrap();
}

const COMMAND_PREFIX: &str = "-";

#[group]
#[commands(pin, unpin)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {}

#[tokio::main]
async fn main() {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix(COMMAND_PREFIX)) // set the bot's prefix
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
fn parse_url(message: &str) -> Option<(u64, u64, u64)> {
    match URL_RE.captures(message) {
        None => None,
        Some(captures) => Some((
            captures.get(1)?.as_str().parse().ok()?,
            captures.get(2)?.as_str().parse().ok()?,
            captures.get(3)?.as_str().parse().ok()?,
        )),
    }
}

async fn send_message_print_err(
    channel_id: ChannelId,
    http: impl AsRef<Http>,
    content: impl std::fmt::Display,
) {
    if let Err(e) = channel_id.say(http, content).await {
        println!("{}", e);
    }
}

async fn process_inputs(strip_prefix: &str, ctx: &Context, msg: &Message) -> Option<Message> {
    match msg.content.strip_prefix(strip_prefix) {
        None => {
            send_message_print_err(msg.channel_id, &ctx.http, "Input required: message URL").await;
            None
        }
        Some(url) => match parse_url(url) {
            None => {
                send_message_print_err(msg.channel_id, &ctx.http, "Invalid message URL").await;
                None
            }
            Some((sv, ch, mg)) => {
                match msg.guild_id {
                    None => {
                        send_message_print_err(
                            msg.channel_id,
                            &ctx.http,
                            "Message sent outside of guild?",
                        )
                        .await;
                        return None;
                    }
                    Some(guild_id) if guild_id != sv => {
                        send_message_print_err(
                            msg.channel_id,
                            &ctx.http,
                            "No. Absolutely not.",
                        )
                        .await;
                        return None;
                    }
                    _ => {}
                }
                match ctx.http.get_message(ch, mg).await {
                    Err(_) => {
                        send_message_print_err(msg.channel_id, &ctx.http, "Couldn't get message")
                            .await;
                        None
                    }
                    Ok(message) => Some(message),
                }
            }
        },
    }
}

#[named]
#[command]
async fn pin(ctx: &Context, msg: &Message) -> CommandResult {
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

    #[test]
    fn good_https() {
        assert_eq!(
            parse_url("https://discord.com/channels/432708847304704010/432708847304704013/1008041568613191813"), 
            Some((432708847304704010, 432708847304704013, 1008041568613191813))
        );
    }

    #[test]
    fn good_http() {
        assert_eq!(
            parse_url("http://discord.com/channels/432708847304704010/432708847304704013/1008041568613191813"), 
            Some((432708847304704010, 432708847304704013, 1008041568613191813))
        );
    }

    #[test]
    fn prefix() {
        assert_eq!(
            parse_url("prefixhttps://discord.com/channels/432708847304704010/432708847304704013/1008041568613191813"),
            None
        );
    }

    #[test]
    fn suffix() {
        assert_eq!(
            parse_url("https://discord.com/channels/432708847304704010/432708847304704013/1008041568613191813suffix"),
            None
        );
    }

    #[test]
    fn protocol() {
        assert_eq!(
            parse_url("ftp://discord.com/channels/432708847304704010/432708847304704013/1008041568613191813"),
            None
        );
    }

    #[test]
    fn endpoint() {
        assert_eq!(
            parse_url("https://discord.com/nonsense/432708847304704010/432708847304704013/1008041568613191813"),
            None
        );
    }

    #[test]
    fn nonsense() {
        assert_eq!(parse_url("not a url at all"), None);
    }

    #[test]
    fn non_numeric() {
        assert_eq!(
            parse_url("https://discord.com/channels/039/not/numeric"),
            None
        );
    }

    #[test]
    fn not_enough_segments() {
        assert_eq!(
            parse_url("https://discord.com/channels/432708847304704010/43"),
            None
        );
    }

    #[test]
    fn empty() {
        assert_eq!(parse_url("https://discord.com/channels///"), None);
    }
}
