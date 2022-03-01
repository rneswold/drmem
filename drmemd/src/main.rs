// Copyright (c) 2020-2022, Richard M Neswold, Jr.
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:
//
// 1. Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in the
//    documentation and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its
//    contributors may be used to endorse or promote products derived
//    from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use drmem_api::{
    driver::{self, API},
    Result,
};
use drmem_config::Config;
use tokio::pin;
use tracing::warn;

mod core;

// If the user specifies the 'grpc' feature, then pull in the module
// that defines the gRPC server.

#[cfg(grpc)]
mod server_grpc;

#[cfg(graphql)]
mod httpd;

// Initializes the `drmemd` application. It determines the
// configuration and sets up the logger. It returns `Some(Config)`
// with the found configuration, if the applications is to run. It
// returns `None` if the program should exit (because a command line
// option asked for a "usage" message, for instance.)

async fn init_app() -> Option<Config> {
    // If a configuration is returned, set up the logger.

    if let Some(cfg) = drmem_config::get().await {
        // Initialize the log system. The max log level is determined
        // by the user (either through the config file or the command
        // line.)

        let subscriber = tracing_subscriber::fmt()
            .with_max_level(cfg.get_log_level())
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("Unable to set global default subscriber");
        Some(cfg)
    } else {
        None
    }
}

// Runs the main body of the application. This top-level task reads
// the config, starts the drivers, and monitors their health.

async fn run() -> Result<()> {
    if let Some(cfg) = init_app().await {
        // Start the core task. It returns a handle to a channel with
        // which to make requests. It also returns the task handle.

        let (tx_drv_req, core_task) = core::start();

        let ctxt = drmem_db_redis::RedisContext::new(
            "sump",
            cfg.get_backend(),
            None,
            None,
        )
        .await?;

        let drv_pump = drmem_drv_sump::Sump::new(
            ctxt,
            &driver::Config::new(),
            driver::RequestChan::new("basement:sump", &tx_drv_req),
        )
        .await?;
        let drv_pump = drv_pump.run();
        pin!(drv_pump);

        #[cfg(all(not(graphql),grpc))]
        {
            tokio::select! {
		Err(e) = core_task => {
                    warn!("core returned: {:?}", e);
		}
		Err(e) = drv_pump => {
                    warn!("monitor returned: {:?}", e);
		}
            }
        }
        #[cfg(all(graphql,not(grpc)))]
        {
            let svr_httpd = httpd::server();
            pin!(svr_httpd);

            tokio::select! {
		Err(e) = core_task => {
                    warn!("core returned: {:?}", e);
		}
		Err(e) = drv_pump => {
                    warn!("monitor returned: {:?}", e);
		}
		Err(e) = svr_httpd => {
                    warn!("httpd returned: {:?}", e);
		}
            }
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("ERROR: {:?}", e)
    }
}
