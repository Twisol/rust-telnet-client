extern crate "rust-telnet" as telnet;

use telnet::carrier::{Carrier};
use telnet::parser::{TelnetTokenizer};
use telnet::dispatch::{DispatchExt};
use telnet::demux::{TelnetDemuxState, ChannelHandler};
use telnet::registry::{EndpointRegistry, TelnetChannel};
use telnet::qstate::{QAttitude};


trait MyWritable {
  fn mywrite(&mut self, _s: String);
}


struct Foo<Parent>(u8);
impl<Parent> TelnetChannel<Parent> for Foo<Parent>
where Parent: MyWritable {
  fn on_data<'a>(&mut self, parent: &mut Parent, channel: Option<u8>, text: &'a [u8]) {
    Carrier{parent: parent, state: self}.on_data(channel, text)
  }
  fn on_command(&mut self, parent: &mut Parent, channel: Option<u8>, command: u8) {
    Carrier{parent: parent, state: self}.on_command(channel, command)
  }
  fn on_enable(&mut self, parent: &mut Parent, channel: Option<u8>) {
    Carrier{parent: parent, state: self}.on_enable(channel)
  }
  fn on_disable(&mut self, parent: &mut Parent, channel: Option<u8>) {
    Carrier{parent: parent, state: self}.on_disable(channel)
  }
  fn on_focus(&mut self, parent: &mut Parent, channel: Option<u8>) {
    Carrier{parent: parent, state: self}.on_focus(channel)
  }
  fn on_blur(&mut self, parent: &mut Parent, channel: Option<u8>) {
    Carrier{parent: parent, state: self}.on_focus(channel)
  }
  fn should_enable(&mut self, parent: &mut Parent, channel: Option<u8>, attitude: QAttitude) -> bool {
    Carrier{parent: parent, state: self}.should_enable(channel, attitude)
  }
}
impl<'parent, 'state, Parent> ChannelHandler for Carrier<'parent, 'state, Parent, Foo<Parent>>
where Parent: MyWritable {
  fn on_data<'a>(&mut self, _channel: Option<u8>, text: &'a [u8]) {
    self.state.0 += 1;
    self.parent.mywrite(format!("[FOO]: {} {}", self.state.0, text));
  }
  fn on_command(&mut self, _channel: Option<u8>, command: u8) {
    self.state.0 += 1;
    self.parent.mywrite(format!("[FOO] {} {}", self.state.0, command));
  }

  fn on_focus(&mut self, _channel: Option<u8>) {
    self.parent.mywrite(format!("[FOO] <focus>"));
  }
  fn on_blur(&mut self, _channel: Option<u8>) {
    self.parent.mywrite(format!("[FOO] <blur>"));
  }
}


struct Main<Parent>;
impl<Parent> TelnetChannel<Parent> for Main<Parent>
where Parent: MyWritable {
  fn on_data<'a>(&mut self, parent: &mut Parent, channel: Option<u8>, text: &'a [u8]) {
    Carrier{parent: parent, state: self}.on_data(channel, text)
  }
  fn on_command(&mut self, parent: &mut Parent, channel: Option<u8>, command: u8) {
    Carrier{parent: parent, state: self}.on_command(channel, command)
  }
  fn on_enable(&mut self, parent: &mut Parent, channel: Option<u8>) {
    Carrier{parent: parent, state: self}.on_enable(channel)
  }
  fn on_disable(&mut self, parent: &mut Parent, channel: Option<u8>) {
    Carrier{parent: parent, state: self}.on_disable(channel)
  }
  fn on_focus(&mut self, parent: &mut Parent, channel: Option<u8>) {
    Carrier{parent: parent, state: self}.on_focus(channel)
  }
  fn on_blur(&mut self, parent: &mut Parent, channel: Option<u8>) {
    Carrier{parent: parent, state: self}.on_focus(channel)
  }
  fn should_enable(&mut self, parent: &mut Parent, channel: Option<u8>, attitude: QAttitude) -> bool {
    Carrier{parent: parent, state: self}.should_enable(channel, attitude)
  }
}
impl<'parent, 'state, Parent> ChannelHandler for Carrier<'parent, 'state, Parent, Main<Parent>>
where Parent: MyWritable {
  fn on_data<'a>(&mut self, _channel: Option<u8>, text: &'a [u8]) {
    self.parent.mywrite(format!("[M]: {}", text));
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

impl MyWritable for () {
  fn mywrite(&mut self, _s: String) {}
}

fn main() {
  let mut output = Output {
    out: String::new(),
  };

  let mut tokenizer = TelnetTokenizer::new();
  let mut demux = TelnetDemuxState::new();

  let mut main_channel = Main;
  let mut foo = Foo(42u8);

  let stream = [
    b"abcdef\xFF\xFA\x20h",
    b"ello",
    b", world!\xFF\xF0\xFF\x42\xFF\xFE\x42"
  ];
  for &data in stream.iter() {
    for token in tokenizer.tokenize(data) {
      // Construct an event handling chain.
      let mut registry = EndpointRegistry::new(&mut output);
      registry.main = Some(&mut main_channel as &mut TelnetChannel<_>);
      registry.endpoints.push(&mut foo);
      registry.command_map.insert(0x42, registry.endpoints.len() - 1);
      registry.channel_map.insert(0x20, registry.endpoints.len() - 1);

      let mut chain = Carrier {
        state: &mut demux,
        parent: &mut registry,
      };

      // Dispatch the event.
      chain.dispatch(token);
    }
  }

  println!("\r\nBuffered output:\r\n{}", output.out);
}
