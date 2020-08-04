use crate::util::auth::AuthUser;
use crate::util::{
    auth::{create_jwt, PrivateClaim},
    cache::{Cache, get, set_exp, delete},
    config::CONFIG,
    discord,
};
use actix_identity::Identity;
use actix_web::{get, http, web, HttpResponse, Responder};
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Serialize, Deserialize)]
struct CallbackQuery {
    code: Option<String>,
    state: Option<String>,
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(me);
    cfg.service(login);
    cfg.service(callback);
    cfg.service(logout);
}

#[get("/@me")]
async fn me(user: AuthUser) -> impl Responder {
    HttpResponse::Ok().json(user.discord)
}

#[get("/login")]
async fn login(cache: Cache) -> impl Responder {
    let state: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .collect();

    let _ = set_exp(cache, &state, "1", "90").await;

    let mut url = Url::parse("https://discord.com/oauth2/authorize").unwrap();
    url.query_pairs_mut()
        .append_pair("state", &state)
        .append_pair("prompt", "none")
        .append_pair("client_id", &CONFIG.discord_id)
        .append_pair("response_type", "code")
        .append_pair("scope", &CONFIG.discord_scopes.join(" "))
        .append_pair("redirect_uri", &CONFIG.discord_redirect);

    HttpResponse::Found()
        .header(http::header::LOCATION, url.to_string())
        .finish()
}

#[get("/callback")]
async fn callback(id: Identity, cache: Cache, query: web::Query<CallbackQuery>) -> impl Responder {
    let query = query.into_inner();
    if query.code.is_none() || query.state.is_none() {
        return HttpResponse::NotAcceptable().body("Not Acceptable").into_body()
    }
    let input_state = query.state.unwrap();

    let state = get(cache.clone(), &input_state).await;
    if state.is_err() {
        return HttpResponse::NotAcceptable().body("Invalid XSSR code, please re-login").into_body()
    }
    let state = state.unwrap();
    warn!("State: {}", &input_state);
    if state.is_empty() {
        return HttpResponse::NotAcceptable().body("Invalid XSSR code, please re-login").into_body()
    }
    let _ = delete(cache, &input_state).await;


    let token_response = discord::fetch_access_token(&query.code.unwrap()).await;
    let token = match token_response {
        Ok(data) => data,
        Err(err) => {
            println!("{:#?}", err);
            return HttpResponse::InternalServerError().body("Internal Server Error")
        }
    };

   let me_response = discord::me(&token.token_type, &token.access_token).await;
   let user = match me_response {
        Ok(data) => data,
        Err(err) => {
            println!("{:#?}", err);
            return HttpResponse::InternalServerError().body("Internal Server Error")
        }
    };

    let private_claim = PrivateClaim::new(user.clone());
    let jwt = create_jwt(private_claim).unwrap();
    id.remember(jwt.clone());

    info!("{} ({}) just logged in!", &user.tag(), &user.id);

    HttpResponse::Found()
        .header(http::header::LOCATION, "/")
        .finish()
}

#[get("/logout")]
async fn logout(id: Identity) -> impl Responder {
    id.forget();

    HttpResponse::Found()
        .header(http::header::LOCATION, "/")
        .finish()
}
