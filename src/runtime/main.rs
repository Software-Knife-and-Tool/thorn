//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! runtime loader/repl
#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

extern crate mu;

#[allow(unused_imports)]
use {
    getopt::Opt,
    mu::{Condition, Mu, Result, System, Tag},
    std::{fs, io::Write},
};

// options
type OptDef = (OptType, String);

#[derive(Debug, PartialEq)]
enum OptType {
    Config,
    Eval,
    Load,
    Pipe,
    Quiet,
}

fn options(mut argv: Vec<String>) -> Option<Vec<OptDef>> {
    let mut opts = getopt::Parser::new(&argv, "h?pvc:e:l:q:");
    let mut optv = Vec::new();

    loop {
        let opt = opts.next().transpose();
        match opt {
            Err(_) => {
                if let Err(error) = opt {
                    eprintln!("runtime: option {error:?}")
                };
                std::process::exit(-1);
            }
            Ok(None) => {
                break;
            }
            Ok(clause) => match clause {
                Some(opt) => match opt {
                    Opt('h', None) | Opt('?', None) => usage(),
                    Opt('v', None) => {
                        print!("runtime: {} ", Mu::VERSION);
                        return None;
                    }
                    Opt('p', None) => {
                        optv.push((OptType::Pipe, String::from("")));
                    }
                    Opt('e', Some(expr)) => {
                        optv.push((OptType::Eval, expr));
                    }
                    Opt('q', Some(expr)) => {
                        optv.push((OptType::Quiet, expr));
                    }
                    Opt('l', Some(path)) => {
                        optv.push((OptType::Load, path));
                    }
                    Opt('c', Some(config)) => {
                        optv.push((OptType::Config, config));
                    }
                    _ => panic!(),
                },
                None => panic!(),
            },
        }
    }

    for file in argv.split_off(opts.index()) {
        optv.push((OptType::Load, file));
    }

    Some(optv)
}

fn usage() {
    println!("runtime: {}: [-h?pvcelq] [file...]", Mu::VERSION);
    println!("?: usage message");
    println!("h: usage message");
    println!("c: [name:value,...]");
    println!("e: eval [form] and print result");
    println!("l: load [path]");
    println!("p: pipe mode");
    println!("q: eval [form] quietly");
    println!("v: print version and exit");

    std::process::exit(0);
}

fn repl(system: &System, _config: &str) {
    let mu = system.mu();

    let eval_string = system
        .eval(&"(mu:open :string :output \"\")".to_string())
        .unwrap();

    let eof_value = system.eval(&"(mu:make-sy \"eof\")".to_string()).unwrap();

    loop {
        match mu.read(mu.std_in(), true, eof_value) {
            Ok(expr) => {
                if mu.eq(expr, eof_value) {
                    break;
                }

                #[allow(clippy::single_match)]
                match mu.compile(expr) {
                    Ok(form) => match mu.eval(form) {
                        Ok(eval) => {
                            mu.write(eval, true, eval_string).unwrap();
                            println!("{}", mu.get_string(eval_string).unwrap());
                        }
                        Err(e) => {
                            eprint!(
                                "eval exception raised by {}, {:?} condition on ",
                                system.write(e.source, true),
                                e.condition
                            );
                            mu.write(e.object, true, mu.err_out()).unwrap();
                            eprintln!()
                        }
                    },
                    Err(e) => {
                        eprint!(
                            "compile exception raised by {}, {:?} condition on ",
                            system.write(e.source, true),
                            e.condition
                        );
                        mu.write(e.object, true, mu.err_out()).unwrap();
                        eprintln!()
                    }
                }
            }
            Err(e) => {
                if let Condition::Eof = e.condition {
                    std::process::exit(0);
                } else {
                    eprint!(
                        "reader exception raised by {}, {:?} condition on ",
                        system.write(e.source, true),
                        e.condition
                    );
                    mu.write(e.object, true, mu.err_out()).unwrap();
                    eprintln!()
                }
            }
        }
    }
}

pub fn main() {
    let mut _config = String::new();
    let mut _debug = false;
    let mut pipe = false;

    match options(std::env::args().collect()) {
        Some(opts) => {
            for opt in opts {
                if opt.0 == OptType::Config {
                    _config = opt.1;
                }
            }
        }
        None => {
            eprintln!("option: error");
            std::process::exit(-1)
        }
    }

    let mu = System::new(String::new());

    match options(std::env::args().collect()) {
        Some(opts) => {
            for opt in opts {
                match opt.0 {
                    OptType::Eval => match mu.eval(&opt.1) {
                        Ok(eval) => println!("{}", mu.write(eval, true)),
                        Err(e) => {
                            eprintln!("runtime: error {}, {}", opt.1, mu.error(e));
                            std::process::exit(-1);
                        }
                    },
                    OptType::Pipe => {
                        pipe = true;
                    }
                    OptType::Load => match mu.load(&opt.1) {
                        Ok(_) => (),
                        Err(e) => {
                            eprintln!("runtime: failed to load {}, {}", &opt.1, mu.error(e));
                            std::process::exit(-1);
                        }
                    },
                    OptType::Quiet => match mu.eval(&opt.1) {
                        Ok(_) => (),
                        Err(e) => {
                            eprintln!("runtime: error {}, {}", opt.1, mu.error(e));
                            std::process::exit(-1);
                        }
                    },
                    OptType::Config => (),
                }
            }
        }
        None => std::process::exit(0),
    };

    if !pipe {
        repl(&mu, "*default*")
    }
}
