use crate::domain::model::FileStatus;
use crate::storage::{file_storage::media_library as disk, postgres::admin_files as db};
use crate::web::error::AppError;
use actix_web::{HttpResponse, web};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::path::Path;
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Default)]
pub struct AdminFiles {
    import: Mutex<Job<ImportReport>>,
    scan: Mutex<Job<ScanReport>>,
}

#[derive(Default, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
enum Job<T> {
    #[default]
    Idle,
    Running,
    Done {
        report: T,
    },
    Failed {
        error: String,
    },
}

#[derive(Default, Serialize)]
struct ImportReport {
    imported: usize,
    skipped: usize,
    failed: usize,
    errors: Vec<String>,
}

impl ImportReport {
    fn fail(&mut self, error: String) {
        self.failed += 1;
        if self.errors.len() < 100 {
            self.errors.push(error);
        }
    }
}

#[derive(Default, Serialize)]
struct ScanReport {
    checked: usize,
    unavailable: Vec<UnavailableFile>,
}

#[derive(Serialize)]
struct UnavailableFile {
    #[serde(flatten)]
    file: db::FilePath,
    reason: String,
}

#[derive(Deserialize)]
pub struct BrowseQuery {
    path: Option<String>,
    #[serde(default)]
    directories_only: bool,
}

#[derive(Deserialize)]
pub struct PathInput {
    path: String,
}

#[derive(Deserialize)]
pub struct RepairInput {
    path: String,
    previous_path: String,
}

fn internal(error: impl std::fmt::Display) -> AppError {
    AppError::internal(format!("admin files: {error:#}"))
}

fn begin<T>(job: &Mutex<Job<T>>) -> Result<(), AppError> {
    let mut job = job.lock().map_err(internal)?;
    if matches!(*job, Job::Running) {
        return Err(AppError::conflict("A job is already running"));
    }
    *job = Job::Running;
    Ok(())
}

fn complete<T>(job: &Mutex<Job<T>>, result: anyhow::Result<T>) {
    if let Ok(mut job) = job.lock() {
        *job = match result {
            Ok(report) => Job::Done { report },
            Err(error) => {
                log::error!("Admin file job failed: {error:#}");
                Job::Failed {
                    error: "The job could not finish. Check the server log and retry.".into(),
                }
            }
        };
    }
}

pub async fn browse(query: web::Query<BrowseQuery>) -> Result<HttpResponse, AppError> {
    let query = query.into_inner();
    let result = web::block(move || disk::browse(query.path.as_deref(), query.directories_only))
        .await
        .map_err(internal)?
        .map_err(|err| AppError::bad_request(format!("{err:#}")))?;
    Ok(HttpResponse::Ok().json(result))
}

pub async fn import_status(state: web::Data<AdminFiles>) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(&*state.import.lock().map_err(internal)?))
}

pub async fn scan_status(state: web::Data<AdminFiles>) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(&*state.scan.lock().map_err(internal)?))
}

pub async fn start_import(
    pool: web::Data<PgPool>,
    state: web::Data<AdminFiles>,
    input: web::Json<PathInput>,
) -> Result<HttpResponse, AppError> {
    let path = input.into_inner().path;
    let directory = web::block(move || disk::import_directory(&path))
        .await
        .map_err(internal)?
        .map_err(|err| AppError::bad_request(format!("{err:#}")))?;
    begin(&state.import)?;
    actix_web::rt::spawn(async move {
        complete(&state.import, run_import(&pool, &directory).await);
    });
    Ok(HttpResponse::Accepted().finish())
}

async fn run_import(pool: &PgPool, directory: &Path) -> anyhow::Result<ImportReport> {
    let directory = directory.to_path_buf();
    let (sender, mut files) = tokio::sync::mpsc::channel(8);
    let producer = tokio::task::spawn_blocking(move || {
        for entry in disk::walk_media(&directory) {
            let prepared = entry.map_err(|err| err.to_string()).and_then(|path| {
                disk::prepare_import(&path).map_err(|err| format!("{}: {err:#}", path.display()))
            });
            if sender.blocking_send(prepared).is_err() {
                break;
            }
        }
    });
    let mut report = ImportReport::default();
    while let Some(file) = files.recv().await {
        match file {
            Ok(file) => {
                let path = file.media.path.clone();
                match db::import(pool, file).await {
                    Ok(true) => report.imported += 1,
                    Ok(false) => report.skipped += 1,
                    Err(error) => {
                        log::error!("Import failed for {path}: {error:#}");
                        report.fail(format!("{path}: could not save to the database"));
                    }
                }
            }
            Err(error) => report.fail(error),
        }
    }
    producer.await?;
    Ok(report)
}

pub async fn start_scan(
    pool: web::Data<PgPool>,
    state: web::Data<AdminFiles>,
) -> Result<HttpResponse, AppError> {
    begin(&state.scan)?;
    actix_web::rt::spawn(async move {
        complete(&state.scan, run_scan(&pool).await);
    });
    Ok(HttpResponse::Accepted().finish())
}

async fn run_scan(pool: &PgPool) -> anyhow::Result<ScanReport> {
    let mut report = ScanReport::default();
    let mut after = Uuid::nil();
    loop {
        let rows = db::paths_after(pool, after).await?;
        let Some(last) = rows.last() else { break };
        after = last.id;
        report.checked += rows.len();
        let mut unavailable = web::block(move || {
            rows.into_iter()
                .filter_map(|file| {
                    disk::availability(&file.path)
                        .err()
                        .map(|error| UnavailableFile {
                            file,
                            reason: format!("{error:#}"),
                        })
                })
                .collect()
        })
        .await?;
        report.unavailable.append(&mut unavailable);
    }
    Ok(report)
}

pub async fn repair(
    pool: web::Data<PgPool>,
    state: web::Data<AdminFiles>,
    id: web::Path<Uuid>,
    input: web::Json<RepairInput>,
) -> Result<HttpResponse, AppError> {
    let id = id.into_inner();
    let input = input.into_inner();
    let previous = db::record(&pool, id)
        .await
        .map_err(internal)?
        .ok_or_else(|| AppError::not_found("File no longer exists in the database"))?;
    if previous.path != input.previous_path
        || matches!(
            FileStatus::from(previous.status),
            FileStatus::Validating | FileStatus::Thumbnailing
        )
    {
        return Err(AppError::conflict(
            "The file changed or is still being processed. Run a new scan.",
        ));
    }

    let previous_path = previous.path.clone();
    let file = web::block(move || {
        if disk::availability(&previous_path).is_ok() {
            return Err(AppError::conflict(
                "The original path is available again. Run a new scan.",
            ));
        }
        disk::inspect_file(Path::new(&input.path))
            .map_err(|error| AppError::bad_request(format!("{error:#}")))
    })
    .await
    .map_err(internal)??;

    if previous
        .hash
        .as_ref()
        .is_some_and(|hash| !hash.eq_ignore_ascii_case(&file.hash))
    {
        return Err(AppError::conflict(
            "The selected file has different contents from the original.",
        ));
    }
    if previous.media_type != i16::from(file.media_type.clone()) {
        return Err(AppError::conflict(
            "The selected file must have the same media type as the original.",
        ));
    }
    if !db::repair(&pool, id, &previous, file)
        .await
        .map_err(internal)?
    {
        return Err(AppError::conflict(
            "The file changed or the selected path is already in use. Run a new scan.",
        ));
    }
    if let Ok(mut scan) = state.scan.lock() {
        if let Job::Done { report } = &mut *scan {
            report
                .unavailable
                .retain(|entry| entry.file.id != id || entry.file.path != previous.path);
        }
    }
    Ok(HttpResponse::NoContent().finish())
}
