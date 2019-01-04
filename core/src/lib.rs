pub enum Response<A, S> {
    Success(A, S, bool),
    Reject(bool),
}

impl<A, S> Response<A, S> {
    pub fn fold<FS, FR, B>(self, success: FS, reject: FR) -> B
        where FS: Fn(A, S, bool) -> B,
              FR: Fn(bool) -> B
    {
        use crate::Response::{Success, Reject};

        match self {
            Success(a, s, b) => success(a, s, b),
            Reject(b) => reject(b)
        }
    }
}

#[cfg(test)]
mod tests_response {
    use crate::Response::{Reject, Success};

    type Response<A> = crate::Response<A, ()>;

    #[test]
    fn it_fold_a_success() {
        let v: Response<u32> = Success(1, (), true);

        assert_eq!(v.fold(|_, _, _| true, |_| false), true);
    }

    #[test]
    fn it_fold_a_reject() {
        let v: Response<u32> = Reject(true);

        assert_eq!(v.fold(|_, _, _| true, |_| false), false);
    }
}

