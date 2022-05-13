// Copyright 2022 Guy Or and the "rtlambda" authors. All rights reserved.

// `SPDX-License-Identifier: MIT OR Apache-2.0`

use std::fmt::{Display, Formatter};

#[derive(Clone, Debug)]
pub struct Error {
    msg: String,
}

impl Error {
    pub fn new(msg: String) -> Self {
        Error { msg }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", &self.msg)
    }
}

pub static CONTAINER_ERR: &str = "Container error. Non-recoverable state.";
