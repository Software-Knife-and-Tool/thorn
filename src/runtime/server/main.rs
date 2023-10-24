//  SPDX-FileCopyrightText: Copyright 2022 James M. Putnam (putnamjm.design@gmail.com)
//  SPDX-License-Identifier: MIT

//! runtime loader/listener
#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

extern crate mu;

#[allow(unused_imports)]
use {
    async_std::net::{IpAddr, Ipv4Addr},
    crossbeam::channel,
    futures::{
        executor::block_on,
        future::BoxFuture,
        task::{self, ArcWake},
    },
    getopt::Opt,
    mu::{Condition, Mu, Result, System, Tag},
    std::{
        cell::RefCell,
        fs,
        future::Future,
        io::Write,
        net::{SocketAddr, ToSocketAddrs},
        pin::Pin,
        sync::{Arc, Mutex},
        task::{Context, Poll, Waker},
        thread,
        time::{Duration, Instant},
    },
};

// options
type OptDef = (OptType, String);

#[derive(Debug, PartialEq)]
enum OptType {
    Config,
    Eval,
    Load,
    Ping,
    Quiet,
    Socket,
}

fn usage() {
    println!("runtime: {}: [-h?psvcelq] [file...]", Mu::VERSION);
    println!("h: usage message");
    println!("?: usage message");
    println!("c: [name:value,...]");
    println!("s: socket [ip-addr:port-number]");
    println!("p: ping mode, requires -s");
    println!("e: eval [form] and print result");
    println!("q: eval [form] quietly");
    println!("l: load [path]");
    println!("v: print version and exit");
    println!("x: ping mode, requires -s");

    std::process::exit(0);
}

fn options(mut argv: Vec<String>) -> Option<Vec<OptDef>> {
    let mut opts = getopt::Parser::new(&argv, "h?s:pvc:e:l:q:");
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
                        optv.push((OptType::Ping, String::from("")));
                    }
                    Opt('e', Some(expr)) => {
                        optv.push((OptType::Eval, expr));
                    }
                    Opt('s', Some(socket)) => {
                        optv.push((OptType::Socket, socket));
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
                    _ => panic!("unmapped switch {}", opt),
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

// 49152 to 65535 are dynamically available
const SERVER_PORT: u16 = 50000;

fn server_options() {
    let mut _config = String::new();
    let mut _ping = false;

    let mut socket = String::new();

    match options(std::env::args().collect()) {
        Some(opts) => {
            for opt in opts {
                if opt.0 == OptType::Config {
                    _config = opt.1.to_string();
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
                    OptType::Config => _config = opt.1.to_string(),
                    OptType::Ping => _ping = true,
                    OptType::Socket => socket = opt.1.to_string(),
                    OptType::Eval => match mu.eval(&opt.1) {
                        Ok(eval) => println!("{}", mu.write(eval, true)),
                        Err(e) => {
                            eprintln!("runtime: error {}, {}", opt.1, mu.error(e));
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
                    OptType::Load => match mu.load(&opt.1) {
                        Ok(_) => (),
                        Err(e) => {
                            eprintln!("runtime: failed to load {}, {}", &opt.1, mu.error(e));
                            std::process::exit(-1);
                        }
                    },
                }
            }
        }
        None => std::process::exit(0),
    };

    if _ping {
        if socket.is_empty() {
            socket = format!("localhost:{}", SERVER_PORT);
        }

        match socket.to_socket_addrs() {
            Ok(mut addrs) => match addrs.next() {
                Some(addr) => {
                    let is_server_port_open =
                        block_on(oports::is_port_open(addr.ip(), addr.port()));

                    println!("server port is {}", is_server_port_open)
                }
                None => {
                    eprintln!("{} is not a legal socket designator", socket);
                    std::process::exit(0)
                }
            },
            Err(_) => {
                eprintln!("cannot resolve host {}", socket);
                std::process::exit(0)
            }
        }
    }
}

//
/// server
//
struct Server {
    scheduled: channel::Receiver<Arc<Task>>,
    sender: channel::Sender<Arc<Task>>,
}

impl Server {
    fn new() -> Self {
        let (sender, scheduled) = channel::unbounded();

        Self { scheduled, sender }
    }

    fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        Task::spawn(future, &self.sender);
    }

    fn run(&self) {
        CURRENT.with(|cell| {
            *cell.borrow_mut() = Some(self.sender.clone());
        });

        while let Ok(task) = self.scheduled.recv() {
            task.poll();
        }
    }
}

pub fn spawn<F>(future: F)
where
    F: Future<Output = ()> + Send + 'static,
{
    CURRENT.with(|cell| {
        let borrow = cell.borrow();
        let sender = borrow.as_ref().unwrap();

        Task::spawn(future, sender);
    });
}

fn _listener(system: &System, _config: &str) {
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

async fn delay(dur: Duration) {
    struct Delay {
        when: Instant,
        waker: Option<Arc<Mutex<Waker>>>,
    }

    impl Future for Delay {
        type Output = ();

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
            if let Some(waker) = &self.waker {
                let mut waker = waker.lock().unwrap();

                if !waker.will_wake(cx.waker()) {
                    *waker = cx.waker().clone();
                }
            } else {
                let when = self.when;
                let waker = Arc::new(Mutex::new(cx.waker().clone()));
                self.waker = Some(waker.clone());

                thread::spawn(move || {
                    let now = Instant::now();

                    if now < when {
                        thread::sleep(when - now);
                    }

                    let waker = waker.lock().unwrap();
                    waker.wake_by_ref();
                });
            }

            if Instant::now() >= self.when {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        }
    }

    let future = Delay {
        when: Instant::now() + dur,
        waker: None,
    };

    future.await;
}

//
// task engine
//
thread_local! {
    static CURRENT: RefCell<Option<channel::Sender<Arc<Task>>>> =
        RefCell::new(None);
}

struct Task {
    future: Mutex<BoxFuture<'static, ()>>,
    executor: channel::Sender<Arc<Task>>,
}

impl Task {
    fn spawn<F>(future: F, sender: &channel::Sender<Arc<Task>>)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let task = Arc::new(Task {
            future: Mutex::new(Box::pin(future)),
            executor: sender.clone(),
        });

        let _ = sender.send(task);
    }

    fn poll(self: Arc<Self>) {
        let waker = task::waker(self.clone());
        let mut cx = Context::from_waker(&waker);
        let mut future = self.future.try_lock().unwrap();

        let _ = future.as_mut().poll(&mut cx);
    }
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        let _ = arc_self.executor.send(arc_self.clone());
    }
}

//
// entry point
//
fn main() {
    let server = Server::new();

    server_options();

    server.spawn(async {
        spawn(async {
            delay(Duration::from_millis(100)).await;
            println!("world");
        });

        spawn(async {
            println!("hello");
        });

        delay(Duration::from_millis(200)).await;
        std::process::exit(0);
    });

    server.run();
}
