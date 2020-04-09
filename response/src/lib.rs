pub enum Response<A, S> {
    Success(A, S),
    Reject,
}

impl<A, S> Response<A, S> {
    pub fn fold<FS, FR, B>(self, success: FS, reject: FR) -> B
    where
        FS: Fn(A, S) -> B,
        FR: Fn() -> B,
    {
        match self {
            Response::Success(a, s) => success(a, s),
            Response::Reject => reject(),
        }
    }
}

#[cfg(test)]
mod tests_response {
    use crate::Response::{Reject, Success};

    type Response<A> = crate::Response<A, ()>;

    #[test]
    fn it_fold_a_success() {
        let v: Response<u32> = Success(1, ());

        assert_eq!(v.fold(|_, _| true, || false), true);
    }

    #[test]
    fn it_fold_a_reject() {
        let v: Response<u32> = Reject;

        assert_eq!(v.fold(|_, _| true, || false), false);
    }
}
