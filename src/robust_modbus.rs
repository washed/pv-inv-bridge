use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio_modbus::{prelude::*, Address, Error as ModbusError, Quantity, Result as ModbusResult};
use tokio_retry::strategy::{jitter, ExponentialBackoff};
use tokio_retry::Retry;

pub(crate) type Coil = bool;
pub(crate) type Word = u16;

#[derive(Debug)]
pub struct RobustContext {
    addr: SocketAddr,
    ctx: client::Context,
    ctx2: Arc<Mutex<std::io::Result<client::Context>>>,
}

impl RobustContext {
    pub async fn new(addr: SocketAddr) -> RobustContext {
        // TODO: get rid of the first context and init the second to an Err/None
        let ctx = tcp::connect_slave(addr, Slave(1)).await.unwrap();
        let ctx2 = tcp::connect_slave(addr, Slave(1)).await.unwrap();

        Self {
            addr,
            ctx,
            ctx2: Arc::new(Mutex::new(Ok(ctx2))),
        }
    }

    pub async fn disconnect(&mut self) -> ModbusResult<()> {
        self.ctx.disconnect().await
    }

    async fn try_read(
        ctx: Arc<Mutex<std::io::Result<client::Context>>>,
        addr: Address,
        cnt: Quantity,
    ) -> ModbusResult<Vec<Word>> {
        let mut ctx_guard = ctx.lock().await;
        let ctx = ctx_guard.as_mut().unwrap();
        let res = ctx.read_input_registers(addr, cnt).await;
        match res {
            Ok(res) => match res {
                Ok(res) => Ok(Ok(res)),
                Err(e) => Ok(Err(e)),
            },
            Err(e) => Err(e),
        }
    }

    async fn set_context(
        ctx: Arc<Mutex<std::io::Result<client::Context>>>,
        addr: SocketAddr,
    ) -> Result<(), ()> {
        let mut ctx_guard = ctx.lock().await;
        println!("trying to connect modbus: {:?}", ctx_guard);
        *ctx_guard = tcp::connect_slave(addr, Slave(1)).await;

        match *ctx_guard {
            Err(_) => Err(()),
            Ok(_) => Ok(()),
        }
    }

    async fn refresh_context(ctx: Arc<Mutex<std::io::Result<client::Context>>>, addr: SocketAddr) {
        let action = || RobustContext::set_context(ctx.clone(), addr);

        let retry_strategy = ExponentialBackoff::from_millis(10)
            .max_delay(Duration::from_secs(5))
            .map(jitter); //.take(10);

        let result = Retry::spawn(retry_strategy, action).await.unwrap();
    }

    fn is_any_err<T>(res: &ModbusResult<T>) -> bool {
        !matches!(res, Ok(Ok(_)))
    }
}

impl SlaveContext for RobustContext {
    fn set_slave(&mut self, slave: Slave) {
        self.ctx.set_slave(slave);
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
        self.ctx.call(request)
    }
}

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
        todo!()
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
        self.ctx.read_discrete_inputs(addr, cnt)
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
        self.ctx.read_holding_registers(addr, cnt)
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
            let action = || async {
                println!("trying to read register");
                let res = RobustContext::try_read(self.ctx2.clone(), addr, cnt).await;
                if res.is_err() {
                    RobustContext::refresh_context(self.ctx2.clone(), self.addr).await;
                }
                res
            };

            let retry_strategy = ExponentialBackoff::from_millis(10)
                .max_delay(Duration::from_secs(1))
                .map(jitter)
                .take(10);

            Retry::spawn(retry_strategy, action).await
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
        self.ctx
            .read_write_multiple_registers(read_addr, read_count, write_addr, write_data)
    }
}
