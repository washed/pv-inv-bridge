use crate::robust_modbus::try_read::TryRead;
use std::io;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};
use tokio_modbus::{prelude::*, Address, Quantity, Result as ModbusResult};
use tokio_retry::strategy::{jitter, FixedInterval};
use tokio_retry::Retry;

use super::{
    Coil, Coils, DiscreteInputs, HoldingRegisters, InputRegisters, ReadWriteMultipleRegisters,
    RobustContext, Word,
};

impl Reader for RobustContext {
    #[doc = " Read multiple coils (0x01)"]
    #[must_use]
    #[allow(
        elided_named_lifetimes,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds
    )]
    fn read_coils<'life0, 'async_trait>(
        &'life0 mut self,
        addr: Address,
        cnt: Quantity,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = ModbusResult<Vec<Coil>>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let action = || async { Coils { addr, cnt }.try_read(self).await };
            Retry::spawn(RobustContext::retry_strategy_command(), action).await
        })
    }

    #[doc = " Read multiple discrete inputs (0x02)"]
    #[must_use]
    #[allow(
        elided_named_lifetimes,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds
    )]
    fn read_discrete_inputs<'life0, 'async_trait>(
        &'life0 mut self,
        addr: Address,
        cnt: Quantity,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = ModbusResult<Vec<Coil>>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let action = || async { DiscreteInputs { addr, cnt }.try_read(self).await };
            Retry::spawn(RobustContext::retry_strategy_command(), action).await
        })
    }

    #[doc = " Read multiple holding registers (0x03)"]
    #[must_use]
    #[allow(
        elided_named_lifetimes,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds
    )]
    fn read_holding_registers<'life0, 'async_trait>(
        &'life0 mut self,
        addr: Address,
        cnt: Quantity,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = ModbusResult<Vec<Word>>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let action = || async { HoldingRegisters { addr, cnt }.try_read(self).await };
            Retry::spawn(RobustContext::retry_strategy_command(), action).await
        })
    }

    #[doc = " Read multiple input registers (0x04)"]
    #[must_use]
    #[allow(
        elided_named_lifetimes,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds
    )]
    fn read_input_registers<'life0, 'async_trait>(
        &'life0 mut self,
        addr: Address,
        cnt: Quantity,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = ModbusResult<Vec<Word>>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let action = || async { InputRegisters { addr, cnt }.try_read(self).await };
            Retry::spawn(RobustContext::retry_strategy_command(), action).await
        })
    }

    #[doc = " Read and write multiple holding registers (0x17)"]
    #[doc = ""]
    #[doc = " The write operation is performed before the read unlike"]
    #[doc = " the name of the operation might suggest!"]
    #[must_use]
    #[allow(
        elided_named_lifetimes,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds
    )]
    fn read_write_multiple_registers<'life0, 'life1, 'async_trait>(
        &'life0 mut self,
        read_addr: Address,
        read_count: Quantity,
        write_addr: Address,
        write_data: &'life1 [Word],
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = ModbusResult<Vec<Word>>>
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
            let action = || async {
                ReadWriteMultipleRegisters {
                    read_addr,
                    read_count,
                    write_addr,
                    write_data,
                }
                .try_read(self)
                .await
            };
            Retry::spawn(RobustContext::retry_strategy_command(), action).await
        })
    }
}
