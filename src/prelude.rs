//! Prelude - Include traits for hal

pub use crate::hal::prelude::*; // embedded hal traits

pub use embedded_hal::digital::v2::OutputPin;

pub use crate::datetime::U32Ext as _stm32wb_hal_datetime_U32Ext;
pub use crate::ipcc::IpccExt as _stm32wb_hal_ipcc_IpccExt;
//pub use crate::dma::DmaExt as _stm32wb_hal_DmaExt;
//pub use crate::flash::FlashExt as _stm32wb_hal_FlashExt;
pub use crate::gpio::GpioExt as _stm32wb_hal_GpioExt;
pub use crate::rcc::RccExt as _stm32wb_hal_RccExt;
pub use crate::time::U32Ext as _stm32wb_hal_time_U32Ext;
