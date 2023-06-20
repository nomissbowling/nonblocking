//! nonblocking stdin

use std::{thread, time};
use std::sync::mpsc;

use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::event; // event::{poll, read};
// use crossterm::event::{Event, KeyCode};

// use crossbeam_channel::{Sender, Receiver, unbounded, tick};

/// NbStdin
pub struct NbStdin {
}

impl NbStdin {

pub fn start() -> Self {
  let nb = NbStdin{};
  enable_raw_mode().unwrap();
  nb
}

pub fn stop(&self) {
  disable_raw_mode().unwrap();
}

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
