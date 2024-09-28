use crate::robust_modbus::try_write::*;
use tokio_modbus::{prelude::*, Address, Result as ModbusResult};
use tokio_retry::Retry;

use super::{Coil, RobustContext, Word};

impl Writer for RobustContext {
    #[doc = " Write a single coil (0x05)"]
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    fn write_single_coil<'life0, 'async_trait>(
        &'life0 mut self,
        addr: Address,
        coil: Coil,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = ModbusResult<()>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let action = || async { CoilWrite { addr, coil }.try_write(self).await };
            Retry::spawn(RobustContext::retry_strategy_command(), action).await
        })
    }

    #[doc = " Write a single holding register (0x06)"]
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    fn write_single_register<'life0, 'async_trait>(
        &'life0 mut self,
        addr: Address,
        word: Word,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = ModbusResult<()>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let action = || async { RegisterWrite { addr, word }.try_write(self).await };
            Retry::spawn(RobustContext::retry_strategy_command(), action).await
        })
    }

    #[doc = " Write multiple coils (0x0F)"]
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    fn write_multiple_coils<'life0, 'life1, 'async_trait>(
        &'life0 mut self,
        addr: Address,
        coils: &'life1 [Coil],
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = ModbusResult<()>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let action = || async { MultipleCoilsWrite { addr, coils }.try_write(self).await };
            Retry::spawn(RobustContext::retry_strategy_command(), action).await
        })
    }

    #[doc = " Write multiple holding registers (0x10)"]
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    fn write_multiple_registers<'life0, 'life1, 'async_trait>(
        &'life0 mut self,
        addr: Address,
        words: &'life1 [Word],
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = ModbusResult<()>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let action = || async { MultipleRegistersWrite { addr, words }.try_write(self).await };
            Retry::spawn(RobustContext::retry_strategy_command(), action).await
        })
    }

    #[doc = " Set or clear individual bits of a holding register (0x16)"]
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    fn masked_write_register<'life0, 'async_trait>(
        &'life0 mut self,
        addr: Address,
        and_mask: Word,
        or_mask: Word,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = ModbusResult<()>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let action = || async {
                RegisterMaskedWrite {
                    addr,
                    and_mask,
                    or_mask,
                }
                .try_write(self)
                .await
            };
            Retry::spawn(RobustContext::retry_strategy_command(), action).await
        })
    }
}
