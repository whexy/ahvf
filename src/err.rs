use crate::bindings::*;

/// An Hypervisor Result.
pub type Result<T> = core::result::Result<T, HypervisorError>;

/// Represent an error returned by the Hypervisor.
#[derive(Copy, Clone, Debug)]
pub enum HypervisorError {
    /// A generic error was returned by the Hypervisor.
    Error,

    /// The Hypervisor is busy.
    Busy,

    /// A bad argument was received.
    BadArgument,

    /// The guest is in an illegal state.
    IllegalGuestState,

    /// No resources availaible.
    NoResources,

    /// No device availaible.
    NoDevice,

    /// Access was denied.
    Denied,

    /// Operation unsupported.
    Unsupported,

    /// Invalid handle sent.
    InvalidHandle,

    /// The given allocation handle is still mapped.
    AllocationStillMapped,

    /// A memory address was misaligned
    MisalignedAddress,

    /// An unknown error was returned.
    Unknown(i32),
}

/// Util used to convert a hv_return_t into a Result
pub fn convert_hv_return(value: hv_return_t) -> Result<()> {
    if value == HV_SUCCESS {
        Ok(())
    } else {
        Err(HypervisorError::from(value))
    }
}

impl From<hv_return_t> for HypervisorError {
    fn from(value: hv_return_t) -> HypervisorError {
        match value {
            HV_SUCCESS => panic!("HV_SUCCESS was not catch beforehand for Result, this is a bug!"),
            HV_ERROR => HypervisorError::Error,
            HV_BUSY => HypervisorError::Busy,
            HV_BAD_ARGUMENT => HypervisorError::BadArgument,
            HV_ILLEGAL_GUEST_STATE => HypervisorError::IllegalGuestState,
            HV_NO_RESOURCES => HypervisorError::NoResources,
            HV_NO_DEVICE => HypervisorError::NoDevice,
            HV_DENIED => HypervisorError::Denied,
            HV_UNSUPPORTED => HypervisorError::Unsupported,
            _ => HypervisorError::Unknown(value),
        }
    }
}
