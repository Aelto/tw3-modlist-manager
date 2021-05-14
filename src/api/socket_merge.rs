use std::{cell::Cell, error::Error, io::{BufRead, Read}, io::BufReader, path::{Path, PathBuf}, process::ChildStdout, sync::{Arc, Mutex}};
use encoding_rs_io::DecodeReaderBytes;
use serde::{Deserialize, Serialize};
use std::thread;
use websocket::{Message, sync::Server};
use websocket::OwnedMessage;

use crate::{constants, models::modlist::ModList};

fn start_merging(modlist_name: &str) -> std::result::Result<BufReader<ChildStdout>, String> {
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
    .arg("--json")
    // .arg("--texteditor")
    // .arg("code")
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

fn get_string_from_file(path: &PathBuf) -> Result<String, Box<dyn Error>> {
  let source_data = std::fs::read(path)?;
  // N.B. `source_data` can be any arbitrary io::Read implementation.
  let mut decoder = DecodeReaderBytes::new(&source_data[..]);

  let mut dest = String::new();
  // decoder implements the io::Read trait, so it can easily be plugged
  // into any consumer expecting an arbitrary reader.
  decoder.read_to_string(&mut dest)?;

  Ok(dest)
}

fn apply_conflict_resolutions(resolution: &SocketMessage) {
  let conflict_start = "<<<<<<< ours";
  let original_start = "||||||| original";
  let original_end = "=======";
  let conflict_end = ">>>>>>> theirs";
  
  let file_path = PathBuf::from(&resolution.file_path);
  let mut content = get_string_from_file(&file_path).unwrap();

  for conflict in &resolution.conflicts {
    // 1.0
    // the context before
    let start = content.find(conflict_start).unwrap() - conflict.context_original_size;
    let end = content.find(conflict_start).unwrap();
    let range = start..end;

    content.replace_range(range, &conflict.context_before);
    
    // 1.1
    // the ours version up until the original version
    let start = content.find(conflict_start).unwrap();
    let end = content.find(original_start).unwrap();
    let range = start..end;

    content.replace_range(range, &conflict.ours);

    // 1.2
    // the original version up until the theirs version
    let start = content.find(original_start).unwrap();
    let end = content.find(original_end).unwrap();
    let range = start..end;

    content.replace_range(range, &conflict.original);

    // 1.3
    // the theirs version up until the context after version
    let start = content.find(original_end).unwrap();
    let end = content.find(conflict_end).unwrap();
    let range = start..end;

    content.replace_range(range, &conflict.theirs);

    // 1.4
    // the context after
    let start = content.find(conflict_end).unwrap();
    let end = start + conflict_end.len() + conflict.context_original_size;
    let range = start..end;

    content.replace_range(range, &conflict.context_after);
  }

  std::fs::write(file_path, content)
  .expect("could not write to the file");
}

pub async fn main(modlist_name: String) {
    let server = Server::bind("127.0.0.1:5001").unwrap();

    println!("listening");

    let modlist_name = Arc::new(modlist_name);

    for request in server.filter_map(Result::ok) {
      let modlist_name = modlist_name.clone();

      if !request.protocols().contains(&"rust-websocket".to_string()) {
        request.reject().unwrap();
        return;
      }

      let mut client = request.use_protocol("rust-websocket").accept().unwrap();

      let ip = client.peer_addr().unwrap();

      println!("Connection from {}", ip);

      let mut reader = start_merging(&modlist_name).expect("could not get the stdout reader for the script merger");
      let (mut receiver, mut sender) = client.split().unwrap();

      thread::spawn(move || {
        reader
        .lines()
        .filter_map(|line| line.ok())
        .for_each(|line| {
          sender.send_message(&OwnedMessage::Text(line))
          .expect("error when sending the message to the client");

        });
      });

      for message in receiver.incoming_messages() {
        let message = message.unwrap();

        match message {
          OwnedMessage::Close(_) => {
            println!("Client {} disconnected", ip);

            return;
          }
          OwnedMessage::Text(message) => {
            let resolutions = serde_json::from_str::<SocketMessage>(&message)
              .expect("error when parsing the socket message");

            apply_conflict_resolutions(&resolutions);
          },
          _ => {},
        }
      }
    }

}

#[derive(Serialize, Deserialize, Debug)]
struct Conflict {
  ours: String,
  original: String,
  theirs: String,

  // some of the code before and after the conflict
  context_before: String,
  context_after: String,
  context_original_size: usize
}

#[derive(Serialize, Deserialize, Debug)]
struct SocketMessage {
  conflicts: Vec<Conflict>,
  file_name: String,
  file_path: String,

  mod_name: String
}