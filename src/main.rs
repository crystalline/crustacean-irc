#![allow(non_snake_case)]

extern crate regex;
use regex::Regex;

use std::net::Ipv4Addr;
use std::str::*;
use std::net::TcpStream;
use std::io::*;
use std::io::{self, BufReader};
use std::thread;
use std::time;
use std::sync::mpsc::channel;

use std::boxed::Box;

static DEBUG: bool = true;

struct IrcClient {
  server: Ipv4Addr,
  port: u16,
  room: String,
  nick: String,
  prefix: String,
  expectedGreeting: String,
  stream: TcpStream,
  reader: BufReader<TcpStream>,
  loginWait: time::Duration,
  handshakeDone: bool,
  msgRegex: Regex
}

struct ChatEvent {
  host: String,
  receiver: String,
  msg: String,
  isPrivate: bool
}

impl IrcClient {
  pub fn new(ip: String, port: String, room: String,nick: String) -> IrcClient {
    let _ip = Ipv4Addr::from_str(&ip).unwrap();
    let _port = port.parse::<u16>().unwrap();
    let stream = TcpStream::connect((_ip, _port)).unwrap();
    let copy = stream.try_clone().unwrap();
    let mut prefix = String::new();
    let reader = BufReader::new(copy);
    prefix.push_str(":");
    prefix.push_str(&nick);
    prefix.push_str("!~irc.rs ");
    let re = Regex::new(&format!(
      "{}{}{}",
      r":([^\s]+)\sPRIVMSG\s",
      r"([^\s]+)",
      r"\s:(.+)")
    ).unwrap();
    let gr = format!("MODE {} :+i",&nick);

    IrcClient {
      server: _ip,
      port: _port,
      room: room,
      nick: nick,
      prefix: prefix,
      stream: stream,
      reader: reader,
      loginWait: time::Duration::from_millis(5000),
      handshakeDone: false,
      msgRegex: re,
      expectedGreeting: gr
    }
  }

  pub fn sendCmd(&mut self, cmd: &str, data: &str) {
    let res;
    let mut to_send = Vec::with_capacity(self.prefix.as_bytes().len() + cmd.as_bytes().len() + data.as_bytes().len());
    to_send.extend_from_slice(self.prefix.as_bytes());
    to_send.extend_from_slice(cmd.as_bytes());
    to_send.extend_from_slice(" ".as_bytes());
    to_send.extend_from_slice(data.as_bytes());
    to_send.extend_from_slice("\n".as_bytes());
    if DEBUG {
      println!("send:{}",String::from_utf8(to_send.clone()).unwrap());
    }
    res = self.stream.write(&to_send);
    self.stream.flush();
  }

  pub fn connectSequence(&mut self) {
    let mut usr_msg = String::new();
    usr_msg.push_str(&self.nick);
    self.sendCmd("NICK",&usr_msg);
    usr_msg.clear();
    thread::sleep(self.loginWait);

    usr_msg.push_str(&self.nick);
    usr_msg.push_str(" 0 * :Real name");
    self.sendCmd("USER", &usr_msg);
    usr_msg.clear();
    thread::sleep(self.loginWait);

    while !self.handshakeDone {
      let res = self.pump_event();
      thread::sleep(time::Duration::from_millis(10));
    }

    usr_msg.push_str(":");
    usr_msg.push_str(&self.room);
    self.sendCmd("JOIN", &usr_msg);
    thread::sleep(self.loginWait);

  }

  pub fn send(&self, msg: String) {

  }

  /*
  pub fn pump_events(&mut self) -> Vec<ChatEvent> {
    let mut ret: Vec<ChatEvent> = Vec::new();
    loop {
      match self.receive() {
        Some(s) => {
          if DEBUG {
            println!("RECV {}", &s)
          }
         let caps = self.msgRegex.captures(&s);
         match caps {
           Some(c) => {
             if c.len() == 3 {
               let host = c[0].to_string();
               let receiver = c[1].to_string();
               let msg = c[2].to_string();
               let isPrivate: bool = receiver == self.nick;
               ret.push( ChatEvent { host, receiver, msg, isPrivate } );
             }
           },
           None => {
           }
         }
      },
        None => {
          println!("None");
          break;
        }
      }
    }
    return ret;
  }
  */

  pub fn pump_event(&mut self) -> Option<ChatEvent> {
    let mut str = String::new();
    let res = self.reader.read_line(&mut str);

    if DEBUG {
      println!("RECV: {}", &str);
    }

    match res {
      Ok(n) => {
        if str.contains(&self.expectedGreeting) {
          self.handshakeDone = true;
        }
        if str.starts_with("PING") {
          let words = str.split(" ").collect::<Vec<&str>>();
          if words.len() >= 2 {
            self.handshakeDone = true;
            self.sendCmd("PONG",words[1]);
          }
          return None;
        } else {
          let c = self.msgRegex.captures(&str);
          match c {
            Some(c) => {
              if c.len() == 3 {
                let host = c[0].to_string();
                let receiver = c[1].to_string();
                let msg = c[2].to_string();
                let isPrivate: bool = receiver == self.nick;
                return Some(ChatEvent { host, receiver, msg, isPrivate });
              }
              return None;
            },
            None => {
              return None;
            }
          }
        }
      },
      Err(e) => {
        return None;
      }
    }
  }
}

fn main() {
  println!("Hello, IRC!");

  let mut client = IrcClient::new(
    /*
    String::from("80.65.57.18"),
    String::from("6667"),
    String::from("#r9kprog"),
    String::from("rustbot")
    */
    String::from("127.0.0.1"),
    String::from("6667"),
    String::from("#main"),
    String::from("rustbot")
  );

  client.connectSequence();

  println!("[CONNECTED]");

  loop {

    if let Some(ev) = client.pump_event() {
      println!("Event: H={} R={} MSG={}", ev.host, ev.receiver, ev.msg);
    }

    thread::sleep(time::Duration::from_millis(10));
  }
}
