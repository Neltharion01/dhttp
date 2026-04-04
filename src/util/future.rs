// Loosely based on futures_lite
// I didn't want to enable macros in tokio nor to enable futures_lite

use std::pin::Pin;
use std::task::{Context, Poll};

use pin_project_lite::pin_project;

pin_project! {
    pub(crate) struct Or<F1, F2> {
        #[pin]
        f1: F1,
        #[pin]
        f2: F2,
    }
}

impl<T, E, F1, F2> Or<F1, F2>
where
    F1: Future<Output = T>,
    F2: Future<Output = E>,
{
    pub(crate) fn new(f1: F1, f2: F2) -> Or<F1, F2> {
        Or { f1, f2 }
    }
}

impl<T, E, F1, F2> Future for Or<F1, F2>
where
    F1: Future<Output = T>,
    F2: Future<Output = E>,
{
    type Output = Result<T, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        if let Poll::Ready(t) = this.f1.poll(cx) {
            return Poll::Ready(Ok(t));
        }
        if let Poll::Ready(t) = this.f2.poll(cx) {
            return Poll::Ready(Err(t));
        }
        Poll::Pending
    }
}
