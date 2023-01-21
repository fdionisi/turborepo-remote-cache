use std::{collections::HashMap, convert::Infallible, net::SocketAddr, sync::Arc};

use hyper::{Body, Request, Response, Server as HyperServer, StatusCode};
use routerify::{prelude::*, Middleware, RequestInfo, Router, RouterService};
use turborepo_core::TurborepoCore;
use url::form_urlencoded;

#[derive(Clone)]
pub struct State {
    core: Arc<TurborepoCore>,
}

fn empty() -> Body {
    Body::empty()
}

fn router(core: &Arc<TurborepoCore>) -> Router<Body, Infallible> {
    Router::builder()
        .data(State { core: core.clone() })
        .middleware(Middleware::pre(logger))
        .head("/v8/artifacts/:id", head)
        .get("/v8/artifacts/:id", get)
        .put("/v8/artifacts/:id", put)
        .post("/v8/artifacts/events", events)
        .err_handler_with_info(error_handler)
        .build()
        .unwrap()
}

// A handler for "/" page.
async fn head(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let artifact_id = req.param("id").unwrap();
    let state = req.data::<State>().unwrap();
    let query = form_urlencoded::parse(req.uri().query().unwrap().as_bytes())
        .into_owned()
        .collect::<HashMap<String, String>>();

    if query.get("slug".into()).is_none() && query.get("teamId".into()).is_none() {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(empty())
            .unwrap());
    }

    let team_id = query
        .get("slug".into())
        .or_else(|| query.get("teamId".into()))
        .unwrap();

    let exists = state
        .core
        .exists_cached_artifact(artifact_id, team_id)
        .await
        .unwrap();

    if !exists {
        return Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(empty())
            .unwrap());
    }

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(empty())
        .unwrap())
}

// A handler for "/" page.
async fn get(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let artifact_id = req.param("id").unwrap();
    let state = req.data::<State>().unwrap();
    let query = form_urlencoded::parse(req.uri().query().unwrap().as_bytes())
        .into_owned()
        .collect::<HashMap<String, String>>();

    if query.get("slug".into()).is_none() && query.get("teamId".into()).is_none() {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(empty())
            .unwrap());
    }

    let team_id = query
        .get("slug".into())
        .or_else(|| query.get("teamId".into()))
        .unwrap();

    match state
        .core
        .get_cached_artifact(artifact_id.to_string(), team_id.to_owned())
        .await
    {
        Ok(artifact) => Ok(Response::builder()
            .status(StatusCode::OK)
            .body(Body::from(artifact))
            .unwrap()),
        Err(_) => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(empty())
            .unwrap()),
    }
}

async fn put(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let artifact_id = req.param("id").unwrap().clone();
    let state = req.data::<State>().to_owned().unwrap().clone();
    let query = form_urlencoded::parse(req.uri().query().unwrap().as_bytes())
        .into_owned()
        .collect::<HashMap<String, String>>();

    if query.get("slug".into()).is_none() && query.get("teamId".into()).is_none() {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(empty())
            .unwrap());
    }

    let team_id = query
        .get("slug".into())
        .or_else(|| query.get("teamId".into()))
        .unwrap();

    state
        .core
        .create_cached_artifact(artifact_id.to_string(), team_id.to_owned(), req.into_body())
        .await
        .unwrap();

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(empty())
        .unwrap())
}

async fn events(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let query = form_urlencoded::parse(req.uri().query().unwrap().as_bytes())
        .into_owned()
        .collect::<HashMap<String, String>>();

    if query.get("slug".into()).is_none() && query.get("teamId".into()).is_none() {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(empty())
            .unwrap());
    }

    let _team_id = query
        .get("slug".into())
        .or_else(|| query.get("teamId".into()))
        .unwrap();

    let body = { hyper::body::to_bytes(req.into_body()).await.unwrap() };

    println!("{}", String::from_utf8(body.to_vec()).unwrap());

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(empty())
        .unwrap())
}

pub struct TurborepoServer {
    core: Arc<TurborepoCore>,
    token: String,
}

impl TurborepoServer {
    pub fn builder() -> TurborepoServerBuilder {
        TurborepoServerBuilder {
            core: None,
            token: None,
        }
    }

    pub async fn listen(&self) -> std::io::Result<()>
// where
    //     A: ToSocketAddrs,
    {
        let addr: SocketAddr = ([127, 0, 0, 1], 3010).into();
        let router = router(&self.core);

        // Create a Service from the router above to handle incoming requests.
        let service = RouterService::new(router).unwrap();

        let server = HyperServer::bind(&addr).serve(service);

        if let Err(e) = server.await {
            eprintln!("server error: {}", e);
        }

        Ok(())
    }
}

pub struct TurborepoServerBuilder {
    core: Option<TurborepoCore>,
    token: Option<String>,
}

impl TurborepoServerBuilder {
    pub fn build(&mut self) -> TurborepoServer {
        TurborepoServer {
            core: Arc::new(self.core.take().expect("can't build without storage")),
            token: self.token.take().expect("can't build without storage"),
        }
    }

    pub fn with_core(&mut self, core: TurborepoCore) -> &mut TurborepoServerBuilder {
        self.core = Some(core);

        self
    }

    pub fn with_token(&mut self, token: String) -> &mut TurborepoServerBuilder {
        self.token = Some(token);

        self
    }
}

async fn logger(req: Request<Body>) -> Result<Request<Body>, Infallible> {
    println!(
        "{} {} {}",
        req.remote_addr(),
        req.method(),
        req.uri().path()
    );

    Ok(req)
}

async fn error_handler(err: routerify::RouteError, _: RequestInfo) -> Response<Body> {
    eprintln!("{}", err);
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::from(format!("Something went wrong: {}", err)))
        .unwrap()
}
