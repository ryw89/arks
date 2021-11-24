use nix::sys::signal;

/// This should be called before calling any cli method or printing any output.
pub fn reset_signal_pipe_handler() -> Result<(), String> {
    #[cfg(target_family = "unix")]
    {
        unsafe {
            signal::signal(signal::Signal::SIGPIPE, signal::SigHandler::SigDfl)
                .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}
