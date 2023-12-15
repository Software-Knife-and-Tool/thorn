//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! mu functions
use crate::{
    core::{
        exception::{self, Condition, Exception},
        frame::Frame,
        mu::Mu,
        types::{Tag, Type},
    },
    system::sys::System,
    types::fixnum::Fixnum,
};

pub trait MuFunction {
    fn sys_exit(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn sys_real_time(_: &Mu, _: &mut Frame) -> exception::Result<()>;
    fn sys_run_time(_: &Mu, _: &mut Frame) -> exception::Result<()>;
}

impl MuFunction for Mu {
    fn sys_real_time(_: &Mu, fp: &mut Frame) -> exception::Result<()> {
        fp.value = match System::real_time() {
            Ok(us) => Fixnum::as_tag(us as i64),
            Err(_) => return Err(Exception::new(Condition::Error, "real-us", Tag::nil())),
        };

        Ok(())
    }

    fn sys_run_time(mu: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let time = mu.start_time.elapsed();
        let usec = time.as_micros();

        fp.value = Fixnum::as_tag(usec.try_into().unwrap());

        Ok(())
    }

    fn sys_exit(_: &Mu, fp: &mut Frame) -> exception::Result<()> {
        let rc = fp.argv[0];

        match Tag::type_of(rc) {
            Type::Fixnum => std::process::exit(Fixnum::as_i64(rc) as i32),
            _ => Err(Exception::new(Condition::Type, "exit", rc)),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn mu_system() {
        assert_eq!(2 + 2, 4);
    }
}
