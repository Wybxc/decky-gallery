use std::{
    cmp::Reverse,
    path::{Component, PathBuf},
};

use argh::FromArgs;
use askama::Template;
use askama_web::WebTemplate;
use poem::{
    EndpointExt, Error, FromRequest, Result, Route, Server,
    error::InternalServerError,
    get, handler,
    http::StatusCode,
    listener::TcpListener,
    middleware::Tracing,
    web::{Path, StaticFileRequest, StaticFileResponse},
};
use wax::walk::Entry;

#[derive(FromArgs)]
/// A simple web server to view Steam screenshots.
struct Args {
    /// the port to listen on
    #[argh(option, short = 'p', default = "3000")]
    port: u16,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();

    let args: Args = argh::from_env();

    let app = Route::new()
        .at("/", get(Index))
        .at("image/*path", get(Image))
        .with(Tracing);
    Server::new(TcpListener::bind(("0.0.0.0", args.port)))
        .run(app)
        .await
}

struct BaseDir(PathBuf);

impl<'a> FromRequest<'a> for BaseDir {
    async fn from_request(req: &'a poem::Request, _body: &mut poem::RequestBody) -> Result<Self> {
        Self::from_request_without_body(req).await
    }

    async fn from_request_without_body(_req: &'a poem::Request) -> Result<Self> {
        match std::env::home_dir() {
            Some(home) => Ok(BaseDir(home.join(".local/share/Steam/userdata"))),
            None => Err(Error::from_string(
                "Could not determine home directory",
                StatusCode::INTERNAL_SERVER_ERROR,
            )),
        }
    }
}

struct ImageEntry {
    path: String,
    thumbnail: String,
}

#[derive(Template, WebTemplate)]
#[template(path = "index.html")]
struct IndexPage {
    images: Vec<ImageEntry>,
}

#[handler]
#[allow(non_snake_case)]
async fn Index(BaseDir(base_dir): BaseDir) -> Result<IndexPage> {
    let glob = wax::Glob::new("*/760/remote/*/screenshots/*.(?i)jpg").unwrap();
    let mut images = glob
        .walk(base_dir)
        .map(|entry| entry.map_err(InternalServerError))
        .collect::<Result<Vec<_>>>()?;
    images
        .sort_by_cached_key(|entry| Reverse(entry.metadata().ok().and_then(|m| m.modified().ok())));
    let images = images
        .into_iter()
        .map(|entry| {
            let path = entry.to_candidate_path().to_string();
            let (folder, filename) = path.rsplit_once("/").unwrap();
            let thumbnail = format!("{folder}/thumbnails/{filename}");
            ImageEntry { path, thumbnail }
        })
        .collect::<Vec<_>>();
    Ok(IndexPage { images })
}

#[handler]
#[allow(non_snake_case)]
async fn Image(
    BaseDir(base_dir): BaseDir,
    Path(path): Path<PathBuf>,
    request: StaticFileRequest,
) -> Result<StaticFileResponse> {
    for component in path.components() {
        match component {
            Component::ParentDir | Component::Prefix(_) | Component::RootDir => {
                return Err(Error::from_string("Invalid path", StatusCode::BAD_REQUEST));
            }
            Component::CurDir | Component::Normal(_) => {}
        }
    }
    if !path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("jpg"))
    {
        return Err(Error::from_string(
            "Invalid file type",
            StatusCode::BAD_REQUEST,
        ));
    }

    let full_path = base_dir.join(path);
    Ok(request.create_response(full_path, true, false)?)
}
