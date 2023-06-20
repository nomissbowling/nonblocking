//! nonblocking stdin

use std::{thread, time};
use std::sync::mpsc;
use std::error::Error;

use futures::StreamExt;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::event; // event::{poll, read};
use crossterm::event::{Event, EventStream, KeyCode, KeyModifiers};
// use crossterm::execute;

// use crossbeam_channel::{Sender, Receiver, unbounded, tick};

/// NbStdin
pub struct NbStdin {
  reader: EventStream
}

impl NbStdin {

/// constructor
pub fn start() -> Self {
  let nb = NbStdin{reader: EventStream::new()};
  enable_raw_mode().unwrap();
  nb
}

/// destructor
pub fn stop(&self) {
  disable_raw_mode().unwrap();
}

/// async-await
pub async fn async_stdin(&mut self) -> Result<bool, Box<dyn Error>> {
  loop {
    match self.reader.next().await {
    None => break,
    Some(Err(e)) => { println!("Error: {:?}", e); break; },
    Some(Ok(e)) => {
      match e {
      Event::Key(ke) => {
        if ke.code == KeyCode::Esc { return Ok(true); }
        if ke.modifiers == KeyModifiers::CONTROL {
          if ke.code == KeyCode::Char('C') { return Ok(true); }
        }
      },
      _ => ()
      }
      break;
    }
    }
  }
  Ok(false)
}

/// another thread
pub fn non_block_stdin(timeout: time::Duration) -> bool {
  let (tx, rx) = mpsc::channel();
  let _handle = thread::spawn(move || {
    loop {
      if !event::poll(timeout).unwrap() { break; } // nonblocking
      else {
        match event::read().unwrap() { // blocking
        e => {
          match tx.send(e) {
          Ok(()) => (),
          Err(_) => ()
          }
          break;
        }
        }
      }
    }
    ()
  });
  match rx.recv_timeout(timeout) {
  Ok(e) => {
    match e {
    _ => ()
    }
  },
  Err(mpsc::RecvTimeoutError::Timeout) => (),
  Err(mpsc::RecvTimeoutError::Disconnected) => unreachable!()
  }
  false
}

}
