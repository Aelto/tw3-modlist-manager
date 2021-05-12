use std::{io::BufRead, io::BufReader, net::SocketAddr, process::ChildStdout, sync::Arc};
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

pub async fn main(modlist_name: String) {
    let server = Server::bind("127.0.0.1:5001").unwrap();

    println!("listening");

    let modlist_name = Arc::new(modlist_name);

    for request in server.filter_map(Result::ok) {
      let modlist_name = modlist_name.clone();

      // Spawn a new thread for each connection.
      thread::spawn(move || {
        if !request.protocols().contains(&"rust-websocket".to_string()) {
          request.reject().unwrap();
          return;
        }
  
        let mut client = request.use_protocol("rust-websocket").accept().unwrap();
  
        let ip = client.peer_addr().unwrap();
  
        println!("Connection from {}", ip);

        let mut reader = start_merging(&modlist_name).expect("could not get the stdout reader for the script merger");
  
        // let message = OwnedMessage::Text("Hello".to_string());
        // client.send_message(&message).unwrap();
  
        let (mut receiver, mut sender) = client.split().unwrap();

        reader
        .lines()
        .filter_map(|line| line.ok())
        .for_each(|line| {
          sender.send_message(&OwnedMessage::Text(line));
        });
  
        for message in receiver.incoming_messages() {
          let message = message.unwrap();
  
          match message {
            OwnedMessage::Close(_) => {
              let message = OwnedMessage::Close(None);
              sender.send_message(&message).unwrap();
              println!("Client {} disconnected", ip);
              return;
            }
            OwnedMessage::Ping(ping) => {
              let message = OwnedMessage::Pong(ping);
              sender.send_message(&message).unwrap();
            }
            _ => sender.send_message(&message).unwrap(),
          }
        }
      });
    }

}