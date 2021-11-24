/// Prints an error message and exits the program.
#[macro_export]
macro_rules! bad_exit {
    ( $e:expr ) => {{
        eprintln!("{}", $e);
        process::exit(1);
    }};
}
