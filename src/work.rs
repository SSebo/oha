pub async fn work<T, F: std::future::Future<Output = T>>(
    task_generator: impl Fn() -> F,
    n_tasks: usize,
    n_workers: usize,
) -> Vec<Vec<T>> {
    let injector = crossbeam::deque::Injector::new();

    for _ in 0..n_tasks {
        injector.push(());
    }

    futures::future::join_all((0..n_workers).map(|_| async {
        let mut ret = Vec::new();
        while let crossbeam::deque::Steal::Success(()) = injector.steal() {
            ret.push(task_generator().await);
        }
        ret
    }))
    .await
}

pub async fn work_with_qps<T, F: std::future::Future<Output = T>>(
    task_generator: impl Fn() -> F,
    qps: usize,
    n_tasks: usize,
    n_workers: usize,
) -> Vec<Vec<T>> {
    let (tx, rx) = crossbeam::channel::unbounded();

    tokio::spawn(async move {
        for _ in 0..n_tasks {
            tx.send(()).unwrap();
            tokio::time::delay_for(std::time::Duration::from_secs(1) / qps as u32).await;
        }
        // tx gone
    });

    futures::future::join_all((0..n_workers).map(|_| async {
        let mut ret = Vec::new();
        while let Ok(()) = rx.recv() {
            ret.push(task_generator().await)
        }
        ret
    }))
    .await
}

pub async fn work_duration<T, F: std::future::Future<Output = T>>(
    task_generator: impl Fn() -> F,
    duration: std::time::Duration,
    n_workers: usize,
) -> Vec<Vec<T>> {
    let start = std::time::Instant::now();
    futures::future::join_all((0..n_workers).map(|_| async {
        let mut ret = Vec::new();
        while (std::time::Instant::now() - start) < duration {
            ret.push(task_generator().await);
        }
        ret
    }))
    .await
}
