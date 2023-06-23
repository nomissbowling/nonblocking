#![doc(html_root_url = "https://docs.rs/nonblock/0.1.4")]
//! nonblocking stdin

pub mod nb_stdin;

#[cfg(test)]
mod tests {
  use super::nb_stdin::NbStdin;
  use futures::executor::block_on;
  use std::time;

  #[test]
  fn a_test() {
    let nb = NbStdin::start();
    let b = NbStdin::non_blocking_stdin(time::Duration::from_millis(20),
      NbStdin::break_with_esc);
    let a = NbStdin::async_non_blocking_stdin(time::Duration::from_millis(20),
      NbStdin::break_with_esc);
    let s = block_on(NbStdin::select_stdin(time::Duration::from_millis(20)));
    nb.stop();
    assert_eq!(b, false);
    assert_eq!(a, false);
    assert_eq!(s.unwrap(), None);
  }
}
