use anyhow::Context;
use clap::Parser;
use tokio::{
    io::{stdout, AsyncReadExt, AsyncWriteExt},
    process::{Child, Command},
    sync::{self, oneshot},
    task::{self, LocalSet},
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(next_line_help = true)]
struct Args {
    #[clap(short = 'c', long = "command", required = true)]
    commands: Vec<String>,
}

struct Job {
    child: Child,
    send_err: sync::mpsc::Sender<()>,
}

impl Job {
    async fn run(mut self, send_output_signal: oneshot::Receiver<()>) {
        let mut job_out = self.child.stdout.take().unwrap();
        let mut job_err = self.child.stderr.take().unwrap();

        let mut out = stdout();

        let mut buf1 = [0; 1028];
        let mut buf2 = [0; 1028];

        send_output_signal.await.unwrap();

        loop {
            tokio::select! {
                Ok(len) = job_out.read (&mut buf1) => {
                    out.write_all(&buf1[..len]).await.unwrap();
                }
                Ok(len) = job_err.read (&mut buf2) => {
                    out.write_all(&buf2[..len]).await.unwrap();
                }
                finished = self.child.wait() => {
                    let mut s = String::new();

                    // TODO: is this sufficient to read rest of output?
                    job_out.read_to_string(&mut s).await.unwrap();

                    out.write_all(s.as_bytes()).await.unwrap();
                    out.flush().await.unwrap();


                    // TODO: is this sufficient to read rest of output?
                    let size = job_err.read_to_string(&mut s).await.unwrap();
                    out.write_all(&s.as_bytes()[..size]).await.unwrap();
                    out.flush().await.unwrap();

                    match finished {
                        Ok(..) => {
                            return;
                        }
                        Err(..) => {
                            self.send_err.send(()).await.unwrap();
                        }
                    }
                }
            }
        }
    }
}

struct Runner {
    jobs: Vec<Job>,
    err_rx: sync::mpsc::Receiver<()>,
}

impl Runner {
    fn new(children: Vec<Child>) -> Self {
        let (err_tx, err_rx) = sync::mpsc::channel(1);

        let jobs: Vec<_> = children
            .into_iter()
            .map(|child| Job {
                child,
                send_err: err_tx.clone(),
            })
            .collect();

        Self { jobs, err_rx }
    }
}

impl Runner {
    async fn run(mut self) {
        let mut send_signals = Vec::new();
        for job in self.jobs {
            let (tx, rx) = oneshot::channel();
            let handle = task::spawn_local(async move {
                job.run(rx).await;
            });
            send_signals.push((handle, tx));
        }

        for (handle, send_output_signal) in send_signals {
            send_output_signal.send(()).unwrap();

            let mut failed = false;

            tokio::select! {
                _ = handle => {},
                _ = self.err_rx.recv() => failed = true,
            }

            if failed {
                return;
            }
        }
    }
}

async fn run() -> anyhow::Result<()> {
    let Args { commands } = Args::parse();

    let mut children = Vec::new();

    for command in commands {
        let (first, xs) = {
            let mut elems = command.split(' ');
            let first = elems.next().expect("empty command");
            (first, elems)
        };

        let cmd = Command::new(first)
            .args(xs)
            .stdout(std::process::Stdio::piped()) // TODO: make not piped
            .stderr(std::process::Stdio::piped())
            .spawn()
            .with_context(|| format!("could not spawn command {command}"))?;

        children.push(cmd);
    }

    let runner = Runner::new(children);

    let ls = LocalSet::new();
    ls.run_until(runner.run()).await;

    Ok(())
}

#[tokio::main]
async fn main() {
    match run().await {
        Ok(..) => {}
        Err(e) => println!("{e}"),
    }
}
