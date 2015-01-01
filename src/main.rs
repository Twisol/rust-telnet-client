#![feature(unboxed_closures)]

extern crate "rust-telnet" as telnet;

use telnet::parser::{TelnetTokenizer};
use telnet::dispatch::{DispatchExt};
use telnet::demux::{TelnetDemuxState, TelnetDemux, ChannelHandler};
use telnet::registry::{EndpointRegistry, TelnetChannel};


trait MyWritable {
  fn mywrite(&mut self, _s: String);
}

struct Foo(u8);
impl<Parent> TelnetChannel<Parent> for Foo
where Parent: MyWritable {
  fn on_data<'a>(&mut self, parent: &mut Parent, _channel: Option<u8>, text: &'a [u8]) {
    self.0 += 1;
    parent.mywrite(format!("[FOO]: {} {}", self.0, text));
  }
  fn on_command(&mut self, parent: &mut Parent, _channel: Option<u8>, command: u8) {
    self.0 += 1;
    parent.mywrite(format!("[FOO] {} {}", self.0, command));
  }

  fn on_focus(&mut self, parent: &mut Parent, _channel: Option<u8>) {
    parent.mywrite(format!("[FOO] <focus>"));
  }
  fn on_blur(&mut self, parent: &mut Parent, _channel: Option<u8>) {
    parent.mywrite(format!("[FOO] <blur>"));
  }
}


struct Main;
impl<Parent> TelnetChannel<Parent> for Main
where Parent: MyWritable {
  fn on_data<'a>(&mut self, parent: &mut Parent, _channel: Option<u8>, text: &'a [u8]) {
    parent.mywrite(format!("[M]: {}", text));
  }
}


struct Output {
  out: String,
}
impl MyWritable for Output {
  fn mywrite(&mut self, s: String) {
    self.out.push_str(&*s);
    self.out.push_str("\r\n");
  }
}
impl ChannelHandler for Output {}

fn main() {
  let mut output = Output {
    out: String::new(),
  };

  let mut tokenizer = TelnetTokenizer::new();

  let mut demux = TelnetDemuxState::new();
  let mut foo = Foo(42);
  let mut main_channel = Main;


  let stream = [
    b"abcdef\xFF\xFA\x20h",
    b"ello",
    b", world!\xFF\xF0\xFF\x42\xFF\xFE\x42"
  ];
  for &data in stream.iter() {
    for token in tokenizer.tokenize(data) {
      // Construct an event context
      let mut registry = EndpointRegistry::new(&mut output);
      registry.main = Some(&mut main_channel as &mut TelnetChannel<_>);
      registry.endpoints.push(&mut foo);
      registry.command_map.insert(0x42, registry.endpoints.len() - 1);
      registry.channel_map.insert(0x20, registry.endpoints.len() - 1);

      let mut demux = TelnetDemux::new(&mut demux, &mut registry);

      // Dispatch the event
      demux.dispatch(token);
    }
  }

  println!("\r\nBuffered output:\r\n{}", output.out);
}
