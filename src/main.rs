use dotenv::dotenv;
use hyper::{body::Bytes, StatusCode};
use once_cell::sync::Lazy;

use std::{collections::HashMap, convert::TryInto, error::Error, fs::{self, File}, process::Command, sync::Arc, time::Duration, thread, io::ErrorKind};

use serenity::{Result as SerenityResult, async_trait, 
    client::{Client, Context, EventHandler}, 
    framework::standard::CommandError, 
    utils::MessageBuilder, 
    framework::{
        StandardFramework,
        standard::{
            Args, CommandResult,
            macros::{command, group, hook},
        },
    }, model::{channel::Message, id::UserId}, prelude::*};

use hyper::{body, Body, Client as HyperClient, Method, Request};
use hyper_tls::HttpsConnector;
use std::{
    self, 
    io::prelude::*,
    env
};

use uuid::Uuid;
use uwuifier::uwuify_str_sse;

use serde::{Serialize, Deserialize};
use songbird::{
    input::{
        self,
        cached::Memory,
    }, SerenityInit,
};

static VOICES: Lazy<HashMap<String, String>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("mario".to_string(), "TM:7wbtjphx8h8v".to_string());
    m.insert("frieza".to_string(), "TM:pgdraamqpbke".to_string());
    m.insert("weirdal".to_string(), "TM:rawayafee2a2".to_string());
    m.insert("moistcr1tikal".to_string(), "TM:k8sj9ghnd3ss".to_string());
    m.insert("sonic".to_string(), "TM:g21ykeqza9zs".to_string());
    m.insert("spongebob".to_string(), "TM:h2x7azcafxhh".to_string());
    m.insert("ben-shapiro".to_string(), "TM:nqwew67rzwz4".to_string());
    m.insert("bernie".to_string(), "TM:vafvg4nskr59".to_string());

    m
});

#[group]
#[commands(join, say, leave, voices, uwu, sus)]
struct General;
struct Handler;

#[async_trait]
impl EventHandler for Handler { }

struct FileLock;

impl TypeMapKey for FileLock {
    type Value = Arc<Mutex<HashMap<UserId, String>>>;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let token = {
        let mut token = String::new();
        for (key, value) in env::vars() {
            if key == "DISCORD_TOKEN" {
                token = value;
            }
        }

        token
    };
    
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("tts_")) // set the bot's prefix to "tts_"
        .after(clear) 
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .register_songbird()
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;
        
        // The FileLock Value has the following type:
        // Arc<RwLock<HashMap<UserId, String>>>
        // So, we have to insert the same type to it.
        data.insert::<FileLock>(Arc::new(Mutex::new(HashMap::new())));
    }

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states.get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            check_msg(msg.reply(ctx, "Not in a voice channel").await);

            return Ok(());
        }
    };

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    let _handler = manager.join(guild_id, connect_to).await;

    Ok(())
}

#[command]
async fn sus(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    if let Some(file) = msg.attachments.first() {
        let ty = args.single::<u32>().unwrap_or(15);
        if ty > 50 {
            check_msg(msg.channel_id.say(ctx, "Please keep dimension under 50").await);
            return Ok(());
        }
        let name = file.filename.clone();
        let downloaded = file.download().await?;
        let mut file = File::create(&name).unwrap();
        let gif = "xd.gif";

        file.write_all(&downloaded).unwrap();

        let _ = Command::new("./among-us-dumpy")
            .args(&[&ty.to_string(), &name, gif])
            .output();

        let files = vec![gif];

        msg.channel_id.send_files(ctx, files, |m| m.content("created gif")).await?;
        fs::remove_file(&name).unwrap();
        fs::remove_file(&gif).unwrap();
            
    } else {
        check_msg(msg.channel_id.say(ctx, "Please give a file").await);
    }
    
    Ok(())
}

#[command]
async fn voices(ctx: &Context, msg: &Message) -> CommandResult {
    let mut voices = String::new();

    for k in VOICES.keys() {
        voices += &format!("\t{}\n", k);
    }

    let response = MessageBuilder::new()
        .push_bold_line("The available voices are:")
        .push(voices)
        .build();

    if let Err(why) = msg.channel_id.say(&ctx.http, &response).await {
        println!("Error sending message: {:?}", why);
    }

    Ok(())
}

#[hook]
async fn clear(ctx: &Context, msg: &Message, cmd_name: &str, _error: Result<(), CommandError>) {
    if cmd_name == "say" {
        let sources_lock = ctx.data
            .read()
            .await
            .get::<FileLock>()
            .cloned()
            .expect("FileLock was installed at startup");

        let mut sources = sources_lock.lock().await;
        
        if let Some(file_name) = sources.get(&msg.author.id) { 
            match fs::remove_file(file_name) {
                Ok(_) => {
                    sources.remove(&msg.author.id);
                },
                Err(_) => {
                    sources.remove(&msg.author.id);
                }
            }
        }
    }
}

#[command]
async fn uwu(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    check_msg(msg.channel_id.say(ctx, uwuify_str_sse(args.rest())).await);
    
    Ok(())
}

#[command]
#[only_in(guilds)]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            check_msg(msg.channel_id.say(&ctx.http, format!("Failed: {:?}", e)).await);
        }

        check_msg(msg.channel_id.say(&ctx.http, "Left voice channel").await);
    } else {
        check_msg(msg.reply(ctx, "Not in a voice channel").await);
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn say(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let guild = msg.guild(&ctx.cache).await.unwrap();
    let guild_id = guild.id;
    let filename = format!("{}.wav", uuid::Uuid::new_v4().to_string());

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        let sources_lock = ctx.data
            .read()
            .await
            .get::<FileLock>()
            .cloned()
            .expect("FileLock was installed at startup");

        let mut sources = sources_lock.lock().await;
        sources.insert(msg.author.id, filename.clone());
        let name = args.single::<String>()?;
        let sentence = args.rest();

        if let Some(passing_name) = VOICES.get(&name) {
            let (code, audio) = get_wav_file(passing_name, sentence)
                .await
                .unwrap_or((StatusCode::BAD_REQUEST, Default::default()));

            if code == StatusCode::OK {
                let mut buffer = File::create(filename.clone())?;
                buffer.write_all(&audio)?;

                let audio = Memory::new(
                    input::ffmpeg(filename.clone())
                        .await
                        .expect("File should be in root folder."),
                    ).expect("These parameters are well-defined.");
                let _ = audio.raw.spawn_loader();
                
                let _song = handler.play_source(audio.try_into().unwrap());
                check_msg(msg.channel_id.say(&ctx.http, "Playing!").await);
            } else {
                check_msg(msg.channel_id.say(&ctx.http, "Couldn't start inference job!").await);
            }
            
        } else {
            check_msg(
                msg.channel_id
                    .say(&ctx.http, format!("{} is not an eligible voice.", name))
                    .await,
            );
        }
    } else {
        check_msg(
            msg.channel_id
                .say(&ctx.http, "Not in a voice channel to play in")
                .await,
        );
    }

    Ok(())
}

fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}

async fn get_wav_file(speaker: &str, text: &str) -> Result<(StatusCode, Bytes), Box<dyn Error>> {
    let uuid = Uuid::new_v4();
    let request = TtsInferenceRequest::new(uuid, speaker, text);
    let req = Request::builder()
        .method(Method::POST)
        .uri("https://api.fakeyou.com/tts/inference")
        .header("content-type", "application/json")
        .header("accept", "application/json")
        .body(Body::from(serde_json::to_string(&request)?))?;

    let https = HttpsConnector::new();
    let client = HyperClient::builder().build::<_, hyper::Body>(https);
    let res = client.request(req).await?;

    let body_bytes = body::to_bytes(res.into_body()).await?;
    let deserialized: TtsInferenceResponse = serde_json::from_str(std::str::from_utf8(&body_bytes)?)?;
    if !deserialized.is_success() {
        return Err(Box::new(std::io::Error::new(ErrorKind::Other, "Job start was not succesful".to_string())));
    }

    let mut code: StatusCode = StatusCode::OK;

    let polled =  loop {
        thread::sleep(Duration::from_secs(3));
        let req = Request::builder()
            .method(Method::GET)
            .uri(format!("https://api.fakeyou.com/tts/job/{}", deserialized.inference_job_token.as_ref().unwrap()))
            .header("accept", "application/json")
            .body(Body::empty())?;
        let res = client.request(req).await?;
        code = res.status();
        let body_bytes = body::to_bytes(res.into_body()).await?;
        let deserialized: TtsJobResult = serde_json::from_str(std::str::from_utf8(&body_bytes)?)?;
        println!("{:#?}", deserialized);
        if deserialized.state.status == "complete_success" {
            break deserialized;
        }
    };
    
    let req = Request::builder()
            .method(Method::GET)
            .uri(format!("https://storage.googleapis.com/vocodes-public{}", polled.state.maybe_public_bucket_wav_audio_path.unwrap()))
            .header("accept", "application/json")
            .body(Body::empty())?;
    let res = client.request(req).await?;
    let code = res.status();
    let body_bytes = body::to_bytes(res.into_body()).await?;
    
    Ok((code, body_bytes))
} 

#[derive(Serialize, Deserialize, Debug)]
struct TtsInferenceRequest<'a> {
    uuid_idempotency_token: Uuid,
    tts_model_token: &'a str,
    inference_text: &'a str
}

impl TtsInferenceRequest<'_> {
    fn new<'a>(uuid_idempotency_token: Uuid, tts_model_token: &'a str, inference_text: &'a str) -> TtsInferenceRequest<'a> {
        TtsInferenceRequest {
            uuid_idempotency_token,
            tts_model_token,
            inference_text
        }
    }    
}

#[derive(Serialize, Deserialize, Debug)]
struct TtsInferenceResponse {
    success: bool,
    inference_job_token: Option<String>
}

impl TtsInferenceResponse {
    fn new() -> TtsInferenceResponse {
        TtsInferenceResponse {
            success: false,
            inference_job_token: None
        }
    }

    fn is_success(&self) -> bool {
        self.success
    }
}
    

#[derive(Serialize, Deserialize, Debug)]
struct State {
    job_token: String,
    status: String,
    maybe_extra_status_description: Option<String>,
    attempt_count: u32,
    maybe_result_token: Option<String>,
    pub maybe_public_bucket_wav_audio_path: Option<String>,
    model_token: String,
    tts_model_type: String,
    title: String,
    raw_inference_text: String,
    created_at: String,
    updated_at: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct TtsJobResult {
    success: bool,
    pub state: State
}