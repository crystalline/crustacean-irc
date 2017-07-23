
use std::net::Ipv4Addr;
use std::str::*;
use std::net::TcpStream;
use std::io::*;
use std::{thread, time};
use std::io::{self, BufReader};

static DEBUG: bool = true;

struct IrcClient {
  server: Ipv4Addr,
  port: u16,
  room: String,
  nick: String,
  prefix: String,
  stream: TcpStream,
  reader: BufReader<TcpStream>,
  loginWait: time::Duration,
  pingDone: bool
}

impl IrcClient {
  pub fn new(ip: String, port: String, room: String,nick: String, onMsg: &Fn(String)) -> IrcClient {
    let _ip = Ipv4Addr::from_str(&ip).unwrap();
    let _port = port.parse::<u16>().unwrap();
    let stream = TcpStream::connect((_ip, _port)).unwrap();
    let copy = stream.try_clone().unwrap();
    let mut prefix = String::new();
    prefix.push_str(":");
    prefix.push_str(&nick);
    prefix.push_str("!~irc.rs ");
    IrcClient {
      server: _ip,
      port: _port,
      room: room,
      nick: nick,
      prefix: prefix,
      stream: stream,
      reader: BufReader::new(copy),
      loginWait: time::Duration::from_millis(5000),
      pingDone: false
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

    self.pr_recv();

    usr_msg.push_str(&self.nick);
    usr_msg.push_str(" 0 * :Real name");
    self.sendCmd("USER", &usr_msg);
    usr_msg.clear();
    thread::sleep(self.loginWait);

    self.pr_recv();

    while !self.pingDone {
      let res = self.receive();
    }

    usr_msg.push_str(":");
    usr_msg.push_str(&self.room);
    self.sendCmd("JOIN", &usr_msg);
    thread::sleep(self.loginWait);

    self.pr_recv();

  }

  pub fn pr_recv(&mut self) {
    match self.receive() {
      Some(s) => {
        println!("received: {}",&s);
      },
      None => {}
    }
  }

  pub fn send(&self, msg: String) {

  }

  pub fn receive(&mut self) -> Option<String> {
    let mut str = String::new();
    let res = self.reader.read_line(&mut str);
    match res {
      Ok(n) => {
        if str.starts_with("PING") {
          let words = str.split(" ").collect::<Vec<&str>>();
          if words.len() >= 2 {
            self.pingDone = true;
            self.sendCmd("PONG",words[1]);
          }
        }
        return Some(str);
      },
      Err(e) => {
        return None;
      }
    }
  }
}

fn main() {
  println!("Hello, IRC!");

  let mut client = IrcClient::new(String::from("80.65.57.18"),String::from("6667"),String::from("#r9kprog"),String::from("rustbot"), &|s: String| {} );

  client.connectSequence();

  println!("[CONNECTED]");

  loop {
    match client.receive() {
      Some(s) => {
        println!("received: {}",&s);
      },
      None => {}
    }
    thread::sleep(time::Duration::from_millis(10));
  }
}
