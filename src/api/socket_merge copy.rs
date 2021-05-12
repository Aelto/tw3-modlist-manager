use std::{io::{BufRead, BufReader}, process::ChildStdout};

use actix::{Actor, StreamHandler};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;

use crate::{constants, models::modlist::ModList};

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

struct MergeActor {
  reader: BufReader<ChildStdout>
}

impl MergeActor {
}

impl Actor for MergeActor {
  type Context = ws::WebsocketContext<Self>;
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MergeActor {
  fn handle(
      &mut self,
      msg: Result<ws::Message, ws::ProtocolError>,
      ctx: &mut Self::Context,
  ) {
      match msg {
          Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
          Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
          Ok(ws::Message::Text(text)) => {
            
            match &text[..] {
              "start" => {
                let mut message = String::new();
                self.reader.read_line(&mut message).expect("error when reading stdout from script merger");

                ctx.text(message);
              }
              _ => {
                println!("received response for conflict = {}", text);
              }
            }

          },
          _ => (),
      }
  }
}

pub async fn index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
  let modlist_name = req.match_info()
    .get("modlist_name")
    .unwrap_or("vanilla");

  let reader = start_merging(&modlist_name).expect("could not setup the merging tool for the modlist");

  let resp = ws::start(MergeActor { reader }, &req, stream);
  println!("{:?}", resp);
  resp
}

fn start_merging(modlist_name: &str) -> Result<BufReader<ChildStdout>, String> {
  let modlist = ModList::get_by_name(modlist_name);

  if modlist.is_none() {
    return Err(String::from("No such modlist"));
  }

  let modlist = modlist.unwrap();

  let scriptmerger_path = std::env::current_dir()
    .unwrap()
    .join(constants::TW3SCRIPTMERGER_PATH);

  let source_path = modlist.content_path();
  let input_path = modlist.mods_path();
  let output_path = input_path
    .join(constants::SCRIPTMERGER_MERGEDFILES_FOLDERNAME)
    .join("content")
    .join("scripts");

  use std::process::{Command, Stdio};
  use std::io::{Error, ErrorKind};

  let stdout = Command::new(scriptmerger_path)
    .arg("--clean")
    .arg("--texteditor")
    .arg("code")
    .arg("--source")
    .arg(&source_path)
    .arg("--input")
    .arg(&input_path)
    .arg("--output")
    .arg(&output_path)
    .stdout(Stdio::piped())
    .spawn()
    .map_err(|err| {
      format!("Internal server error: could not run the tw3-script-merger tool. {}", err)
    })?
    .stdout
    .ok_or_else(|| Error::new(ErrorKind::Other, "Could not capture standard output"))
    .map_err(|err| {
      format!("Internal server error: an error occured when listening to the tw3-script-merger tool. {}", err)
    })?;

  Ok(BufReader::new(stdout))
}