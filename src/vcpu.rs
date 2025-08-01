use crate::bindings::*;
use crate::err::{HypervisorError, Result, convert_hv_return};
use crate::reg::*;
use core::ffi::c_void;

/// Cache type.
#[derive(Copy, Clone, Debug)]
pub enum CacheType {
    /// Data cache.
    Data,

    /// Instruction cache.
    Instruction,
}

impl From<CacheType> for hv_cache_type_t {
    fn from(value: CacheType) -> hv_cache_type_t {
        match value {
            CacheType::Data => hv_cache_type_t_HV_CACHE_TYPE_DATA,
            CacheType::Instruction => hv_cache_type_t_HV_CACHE_TYPE_INSTRUCTION,
        }
    }
}

/// vCPU configuration for a Virtual Machine.
#[derive(Debug)]
pub struct VirtualCpuConfiguration {
    /// Handle of the vCPU configuration.
    pub handle: hv_vcpu_config_t,
}

impl VirtualCpuConfiguration {
    /// Create a new vCPU configuration.
    pub fn new() -> Self {
        VirtualCpuConfiguration {
            handle: unsafe { hv_vcpu_config_create() },
        }
    }

    /// Return value of a feature register.
    pub fn get_feature_register(&self, feature_register: FeatureRegister) -> Result<u64> {
        let mut result = 0;

        let ret = unsafe {
            hv_vcpu_config_get_feature_reg(
                self.handle,
                hv_feature_reg_t::from(feature_register),
                &mut result as *mut u64,
            )
        };

        // Ensure no error got reported
        convert_hv_return(ret)?;

        Ok(result)
    }

    /// Return values of CCSIDR_EL1 for a given cache type.
    pub fn get_ccsidr_el1_sys_register_values(&self, cache_type: CacheType) -> Result<[u64; 8]> {
        let mut result = [0x0; 8];

        let ret = unsafe {
            hv_vcpu_config_get_ccsidr_el1_sys_reg_values(
                self.handle,
                hv_cache_type_t::from(cache_type),
                &mut result[0] as *mut u64,
            )
        };

        // Ensure no error got reported
        convert_hv_return(ret)?;

        Ok(result)
    }
}

unsafe extern "C" {
    fn os_release(object: *mut c_void);
}

impl Drop for VirtualCpuConfiguration {
    fn drop(&mut self) {
        unsafe {
            os_release(self.handle as *mut c_void);
        }
    }
}

/// ARM interrupt type.
#[derive(Copy, Clone, Debug)]
pub enum InterruptType {
    /// ARM IRQ.
    IRQ,

    /// ARM FIQ.
    FIQ,
}

impl From<InterruptType> for hv_interrupt_type_t {
    fn from(value: InterruptType) -> hv_interrupt_type_t {
        match value {
            InterruptType::IRQ => hv_interrupt_type_t_HV_INTERRUPT_TYPE_IRQ,
            InterruptType::FIQ => hv_interrupt_type_t_HV_INTERRUPT_TYPE_FIQ,
        }
    }
}

#[derive(Copy, Clone, Debug)]
/// Exit reason of a vCPU.
pub enum VirtualCpuExitReason {
    /// Asynchronous exit.
    Cancelled,

    /// Guest exception.
    Exception {
        /// The informations about the guest exception.
        exception: hv_vcpu_exit_exception_t,
    },

    /// Virtual Timer enters the pending state.
    VTimerActivated,

    /// Unexpected exit.
    Unknown,
}

impl From<hv_vcpu_exit_t> for VirtualCpuExitReason {
    fn from(value: hv_vcpu_exit_t) -> VirtualCpuExitReason {
        match value.reason {
            hv_exit_reason_t::HV_EXIT_REASON_CANCELED => VirtualCpuExitReason::Cancelled,
            hv_exit_reason_t::HV_EXIT_REASON_EXCEPTION => VirtualCpuExitReason::Exception {
                exception: value.exception,
            },
            hv_exit_reason_t::HV_EXIT_REASON_VTIMER_ACTIVATED => {
                VirtualCpuExitReason::VTimerActivated
            }
            hv_exit_reason_t::HV_EXIT_REASON_UNKNOWN => VirtualCpuExitReason::Unknown,

            // Unexpected unknown
            _ => VirtualCpuExitReason::Unknown,
        }
    }
}

/// vCPU for a Virtual Machine.
#[derive(Debug)]
pub struct VirtualCpu {
    /// Handle of the vCPU configuration.
    pub handle: hv_vcpu_t,

    /// vCPU exit informations.
    pub vcpu_exit: *const hv_vcpu_exit_t,
}

impl Drop for VirtualCpu {
    fn drop(&mut self) {
        self.exit().expect("Cannot exit vCPU on drop!");

        let ret = unsafe { hv_vcpu_destroy(self.handle) };

        convert_hv_return(ret).expect("Cannot destroy vCPU on drop!")
    }
}

impl VirtualCpu {
    /// Gets vCPU handle.
    pub fn get_handle(&self) -> hv_vcpu_t {
        self.handle
    }

    /// Gets a register value.
    ///
    /// **This should be called in the thread that will run the vCPU as it's resident inside it.**
    pub fn get_register(&mut self, register: Register) -> Result<u64> {
        let mut result = 0;

        let ret = unsafe {
            hv_vcpu_get_reg(
                self.handle,
                hv_reg_t::from(register),
                &mut result as *mut u64,
            )
        };

        // Ensure no error got reported
        convert_hv_return(ret)?;

        Ok(result)
    }

    /// Sets a register value.
    ///
    /// **This should be called in the thread that will run the vCPU as it's resident inside it.**
    pub fn set_register(&mut self, register: Register, value: u64) -> Result<()> {
        let ret = unsafe { hv_vcpu_set_reg(self.handle, hv_reg_t::from(register), value) };

        convert_hv_return(ret)
    }

    // TODO: SIMD APIs

    /// Gets a system register value.
    ///
    /// **This should be called in the thread that will run the vCPU as it's resident inside it.**
    pub fn get_system_register(&mut self, register: SystemRegister) -> Result<u64> {
        let mut result = 0;

        let ret = unsafe {
            hv_vcpu_get_sys_reg(
                self.handle,
                hv_sys_reg_t::from(register),
                &mut result as *mut u64,
            )
        };

        // Ensure no error got reported
        convert_hv_return(ret)?;

        Ok(result)
    }

    /// Sets a system register value.
    ///
    /// **This should be called in the thread that will run the vCPU as it's resident inside it.**
    pub fn set_system_register(&mut self, register: SystemRegister, value: u64) -> Result<()> {
        let ret = unsafe { hv_vcpu_set_sys_reg(self.handle, hv_sys_reg_t::from(register), value) };

        convert_hv_return(ret)
    }

    /// Gets pending interrupts.
    ///
    /// **This should be called in the thread that will run the vCPU as it's resident inside it.**
    pub fn get_pending_interrupt(&mut self, interrupt_type: InterruptType) -> Result<bool> {
        let mut result = false;

        let ret = unsafe {
            hv_vcpu_get_pending_interrupt(
                self.handle,
                hv_interrupt_type_t::from(interrupt_type),
                &mut result,
            )
        };

        convert_hv_return(ret)?;

        Ok(result)
    }

    /// Sets pending interrupts.
    ///
    /// **This should be called in the thread that will run the vCPU as it's resident inside it.**
    /// **Pending interrupts automatically get cleared after vCPU run and must be resetup before every call to run.**
    pub fn set_pending_interrupt(
        &mut self,
        interrupt_type: InterruptType,
        value: bool,
    ) -> Result<()> {
        let ret = unsafe {
            hv_vcpu_set_pending_interrupt(
                self.handle,
                hv_interrupt_type_t::from(interrupt_type),
                value,
            )
        };

        convert_hv_return(ret)
    }

    /// Gets whether debug exceptions exit the vCPU.
    ///
    /// **This should be called in the thread that will run the vCPU as it's resident inside it.**
    pub fn get_trap_debug_exceptions(&mut self) -> Result<bool> {
        let mut result = false;

        let ret = unsafe { hv_vcpu_get_trap_debug_exceptions(self.handle, &mut result) };

        convert_hv_return(ret)?;

        Ok(result)
    }

    /// Gets whether debug exceptions exit the vCPU.
    ///
    /// **This should be called in the thread that will run the vCPU as it's resident inside it.**
    pub fn set_trap_debug_exceptions(&mut self, value: bool) -> Result<()> {
        let ret = unsafe { hv_vcpu_set_trap_debug_exceptions(self.handle, value) };

        convert_hv_return(ret)
    }

    /// Gets whether debug-register accesses exit the vCPU.
    ///
    /// **This should be called in the thread that will run the vCPU as it's resident inside it.**
    pub fn get_trap_debug_reg_accesses(&mut self) -> Result<bool> {
        let mut result = false;

        let ret = unsafe { hv_vcpu_get_trap_debug_reg_accesses(self.handle, &mut result) };

        convert_hv_return(ret)?;

        Ok(result)
    }

    /// Gets whether debug-register accesses exit the vCPU.
    ///
    /// **This should be called in the thread that will run the vCPU as it's resident inside it.**
    pub fn set_trap_debug_reg_accesses(&mut self, value: bool) -> Result<()> {
        let ret = unsafe { hv_vcpu_set_trap_debug_reg_accesses(self.handle, value) };

        convert_hv_return(ret)
    }

    /// Runs the vCPU.
    ///
    /// **This should be called in the thread that will run the vCPU as it's resident inside it.**
    pub fn run(&mut self) -> Result<VirtualCpuExitReason> {
        let ret = unsafe { hv_vcpu_run(self.handle) };

        convert_hv_return(ret)?;

        Ok(VirtualCpuExitReason::from(unsafe { *self.vcpu_exit }))
    }

    /// Forces exit the vCPU.
    pub fn exit(&mut self) -> Result<()> {
        let ret = unsafe { hv_vcpus_exit(&mut self.handle, 1) };

        convert_hv_return(ret)
    }

    /// Gets cumulative execution time of a vCPU in mach_absolute_time().
    ///
    /// **This should be called in the thread that will run the vCPU as it's resident inside it.**
    pub fn get_exec_time(&mut self) -> Result<u64> {
        let mut result = 0;

        let ret = unsafe { hv_vcpu_get_exec_time(self.handle, &mut result) };

        convert_hv_return(ret)?;

        Ok(result)
    }

    /// Gets Virtual Timer mask.
    pub fn get_vtimer_mask(&mut self) -> Result<bool> {
        let mut result = false;

        let ret = unsafe { hv_vcpu_get_vtimer_mask(self.handle, &mut result) };

        convert_hv_return(ret)?;

        Ok(result)
    }

    /// Sets Virtual Timer mask.
    pub fn set_vtimer_mask(&mut self, value: bool) -> Result<()> {
        let ret = unsafe { hv_vcpu_set_vtimer_mask(self.handle, value) };

        convert_hv_return(ret)
    }

    /// Gets Virtual Timer offset (CNTVOFF_EL2).
    pub fn get_vtimer_offset(&mut self) -> Result<u64> {
        let mut result = 0;

        let ret = unsafe { hv_vcpu_get_vtimer_offset(self.handle, &mut result) };

        convert_hv_return(ret)?;

        Ok(result)
    }

    /// Sets Virtual Timer offset (CNTVOFF_EL2).
    pub fn set_vtimer_offset(&mut self, value: u64) -> Result<()> {
        let ret = unsafe { hv_vcpu_set_vtimer_offset(self.handle, value) };

        convert_hv_return(ret)
    }
}
