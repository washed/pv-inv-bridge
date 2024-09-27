mod reader;
mod registers;
mod try_read;

use crate::robust_modbus::registers::*;
use std::io;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};
use tokio_modbus::{prelude::*, Result as ModbusResult};
use tokio_retry::strategy::{jitter, FixedInterval};
use tokio_retry::Retry;

pub(crate) type Coil = bool;
pub(crate) type Word = u16;

#[derive(Debug)]
pub struct RobustContext {
    socket_address: SocketAddr,
    slave: Slave,
    slave_sender: mpsc::Sender<Slave>,
    ctx: Arc<Mutex<io::Result<client::Context>>>,
}

impl RobustContext {
    pub async fn new(host: &str, slave: Slave) -> io::Result<RobustContext> {
        let socket_address: SocketAddr = host.to_socket_addrs()?.next().ok_or(io::Error::new(
            io::ErrorKind::AddrNotAvailable,
            "cannot resolve hostname",
        ))?;

        let ctx = Arc::new(Mutex::new(Err(io::Error::new(
            io::ErrorKind::NotConnected,
            "not yet connected",
        ))));

        let (slave_sender, mut slave_receiver) = mpsc::channel(8);

        let ctx_slave_setter = ctx.clone();
        tokio::spawn(async move {
            tokio::select! {
                Some(slave) = slave_receiver.recv() => {
                    let _ = Self::set_slave(ctx_slave_setter, slave).await;
                }
            }
        });

        Ok(Self {
            socket_address,
            slave,
            slave_sender,
            ctx,
        })
    }

    fn retry_strategy_connect() -> impl Iterator<Item = Duration> {
        FixedInterval::from_millis(10).map(jitter).take(3)
    }

    fn retry_strategy_command() -> impl Iterator<Item = Duration> {
        FixedInterval::from_millis(10).map(jitter).take(3)
    }

    pub async fn disconnect(&mut self) -> ModbusResult<()> {
        let mut ctx_guard = self.ctx.lock().await;
        let ctx = ctx_guard
            .as_mut()
            .map_err(|e| io::Error::new(e.kind(), e.to_string()))?;

        ctx.disconnect().await
    }

    async fn set_context(
        ctx: Arc<Mutex<io::Result<client::Context>>>,
        addr: SocketAddr,
        slave: Slave,
    ) -> Result<(), ()> {
        let mut ctx_guard = ctx.lock().await;
        println!("trying to connect modbus: {:?}", ctx_guard);
        *ctx_guard = tcp::connect_slave(addr, slave).await;

        match *ctx_guard {
            Err(_) => Err(()),
            Ok(_) => Ok(()),
        }
    }

    async fn set_slave(
        ctx: Arc<Mutex<io::Result<client::Context>>>,
        slave: Slave,
    ) -> io::Result<()> {
        let mut ctx_guard = ctx.lock().await;
        let ctx = ctx_guard
            .as_mut()
            .map_err(|e| io::Error::new(e.kind(), e.to_string()))?;
        ctx.set_slave(slave);

        Ok(())
    }

    async fn refresh_context(
        ctx: Arc<Mutex<io::Result<client::Context>>>,
        addr: SocketAddr,
        slave: Slave,
    ) {
        let action = || RobustContext::set_context(ctx.clone(), addr, slave);
        match Retry::spawn(RobustContext::retry_strategy_connect(), action).await {
            Ok(_) => println!("successfully reconnected modbus"),
            Err(_) => println!("could not reconnect modbus"),
        };
    }
}

impl SlaveContext for RobustContext {
    fn set_slave(&mut self, slave: Slave) {
        self.slave_sender.blocking_send(slave);
    }
}

impl Client for RobustContext {
    #[doc = " Invoke a _Modbus_ function"]
    #[must_use]
    #[allow(
        elided_named_lifetimes,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds
    )]
    fn call<'life0, 'life1, 'async_trait>(
        &'life0 mut self,
        request: Request<'life1>,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = ModbusResult<Response>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async {
            let mut ctx_guard = self.ctx.lock().await;
            let ctx = ctx_guard
                .as_mut()
                .map_err(|e| io::Error::new(e.kind(), e.to_string()))?;

            ctx.call(request).await
        })
    }
}
