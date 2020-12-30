use std::io;
use std::str;
use std::{
    pin::Pin,
    task::{Context, Poll, Waker},
};

use hyper::client::connect::{Connected, Connection};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

pub struct MockPollStream {
    data: Vec<u8>,
    pos: usize,
    ready_for_response: bool,
    waker: Option<Waker>,
}

impl MockPollStream {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            pos: 0,
            ready_for_response: false,
            waker: None,
        }
    }
}

impl AsyncRead for MockPollStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        if !self.ready_for_response {
            trace!("Not ready for read yet");
            self.waker = Some(cx.waker().clone());
            return Poll::Pending;
        }
        trace!("Data size: {}, Pos: {}", self.data.len(), self.pos);

        buf.put_slice(&self.data[self.pos..]);
        self.waker = Some(cx.waker().clone());
        Poll::Ready(Ok(()))
    }
}

impl AsyncWrite for MockPollStream {
    fn poll_write(self: Pin<&mut Self>, _cx: &mut Context, data: &[u8]) -> Poll<io::Result<usize>> {
        trace!(
            "Request data: {}",
            str::from_utf8(data).unwrap_or("<bad utf-8>")
        );
        let Self {
            ready_for_response,
            waker,
            ..
        } = self.get_mut();
        *ready_for_response = true;
        waker.take().map(|w| w.wake());
        Poll::Ready(Ok(data.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

impl Connection for MockPollStream {
    fn connected(&self) -> Connected {
        Connected::new()
    }
}
