#[macro_export]
macro_rules! simple_error_result {
    // ($err_msg:expr) => {{
        //     Err(Box::new(simple_error::SimpleError::new($err_msg)))
        // }};
        ($fmt:literal $(, $x:expr )*) => {{
            Err(Box::new(simple_error::SimpleError::new(format!($fmt, $($x),*))))
        }};
    }

#[macro_export]
macro_rules! path_as_str {
    ($p:expr) => {
        $p.to_str().unwrap_or("")
    };
}

// #[macro_export]
// macro_rules! path_get_filename {
//     ($p:expr) => {
//         $p.file_name().into_string().unwrap_or_else(|_| "".to_string())
//     };
// }

#[cfg(test)]
mod tests {
    use crate::simple_error_result;
    use simple_error::SimpleError;
    #[test]
    fn simple_error_result() {
        let e1: Result<(), Box<SimpleError>> = simple_error_result!("some_error");
        let e2: Result<(), Box<SimpleError>> = simple_error_result!("{}-{}", 5, 3);

        if let Err(err) = e1 {
            assert_eq!((*err).as_str(), "some_error");
        }

        if let Err(err) = e2 {
            assert_eq!((*err).as_str(), "5-3");
        }
    }
}
