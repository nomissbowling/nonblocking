//! nonblocking stdin

use std::{thread, time};
use std::sync::mpsc;
use std::error::Error;

use futures::{select, pin_mut, future::FutureExt};
use futures::StreamExt;
use futures::executor::block_on;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::event; // event::{poll, read};
use crossterm::event::{Event, EventStream, KeyCode, KeyModifiers};
// use crossterm::execute;

// use crossbeam_channel::{Sender, Receiver, unbounded, tick};

/// NbStdin
pub struct NbStdin {
  reader: EventStream
}

/// NbStdin
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

/// default for async_stdin
pub fn break_with_esc(e: Event) -> bool {
  match e {
  Event::Key(ke) => {
    if ke.code == KeyCode::Esc { return true; }
    if ke.modifiers == KeyModifiers::CONTROL {
      if ke.code == KeyCode::Char('c') { return true; } // ^c
    }
    if ke.modifiers == KeyModifiers::CONTROL | KeyModifiers::SHIFT {
      if ke.code == KeyCode::Char('C') { return true; } // ^c with shift
    }
    ()
  },
  _ => ()
  }
  false
}

/// async-await
pub async fn async_stdin(&mut self, lambda: fn (e: Event) -> bool) -> Result<bool, Box<dyn Error>> {
  match self.reader.next().await { // blocking
  None => (),
  Some(Err(e)) => { println!("Error: {:?}", e); () },
  Some(Ok(e)) => { if lambda(e) { return Ok(true); } }
  }
  Ok(false)
}

/// select async-await
pub async fn select_stdin() -> Result<Option<Event>, Box<dyn Error>> {
  let mut reader = EventStream::new();
  loop {
    let c = reader.next().fuse();
    pin_mut!(c);
    select! {
      e = c => {
        match e {
        None => (),
        Some(Err(e)) => { println!("Error: {:?}", e); () },
        Some(Ok(e)) => { return Ok(Some(e)); }
        }
      },
      complete => break,
      default => break // or () to block in the loop or unreachable!()
    }
  }
  Ok(None)
}

/// async another thread
pub fn async_non_blocking_stdin(timeout: time::Duration, lambda: fn (e: Event) -> bool) -> bool {
  let (tx, rx) = mpsc::channel();
  let _handle = thread::spawn(move || {
    loop {
      match block_on(NbStdin::select_stdin()).unwrap() { // blocking
      None => (),
      Some(e) => {
        match tx.send(e) {
        Ok(()) => break,
        Err(_) => ()
        }
      }
      }
    }
    ()
  });
  match rx.recv_timeout(timeout) {
  Ok(e) => { if lambda(e) { return true; } },
  Err(mpsc::RecvTimeoutError::Timeout) => (),
  Err(mpsc::RecvTimeoutError::Disconnected) => { println!("disconnected"); () } // unreachable!()
  }
  false
}

/// another thread
pub fn non_blocking_stdin(timeout: time::Duration, lambda: fn (e: Event) -> bool) -> bool {
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
  Ok(e) => { if lambda(e) { return true; } },
  Err(mpsc::RecvTimeoutError::Timeout) => (),
  Err(mpsc::RecvTimeoutError::Disconnected) => { println!("disconnected"); () } // unreachable!()
  }
  false
}

}
