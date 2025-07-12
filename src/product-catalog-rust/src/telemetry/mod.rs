// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

pub mod logs_conf;
pub mod resources_conf;
pub mod traces_conf;

pub use logs_conf::init_logger;
pub use resources_conf::get_resource_attr;
pub use traces_conf::{init_reqwest_tracing, init_tracer};