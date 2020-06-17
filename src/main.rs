use tera::{Tera, Context};
use actix_web::{Responder, App, HttpServer, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::fs::File;
use std::time::SystemTime;
use std::io::{Error, Write, Read};
use serde_json::json;


struct AppState {
    templates: Tera,
}

#[derive(Deserialize, Serialize, Debug)]
struct PostData {
    user: String,
    exp: u32,
    pwd: String,
    fname: String,
    desc: String,
    lang: String,
    code: String,
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let tera = Tera::new("templates/*.html").unwrap();
        App::new().data(AppState { templates: tera })
            .service(web::resource("/").route(web::get().to(index)))
            .service(web::resource("/paste").route(web::get().to(paste_main))
                .route(web::post().to(post_data)))
            .service(web::resource("/paste/{page_id}")
                .route(web::get().to(get_page)))
            .service(web::resource("/error").route(web::to(error_page)))
    }).bind("127.0.0.1:7000")?.run().await
}

async fn index() -> impl Responder {
    HttpResponse::PermanentRedirect().header("location", "/paste").body("")
}

async fn paste_main(data: web::Data<AppState>, _req: HttpRequest) -> impl Responder {
    let mut ctx = Context::new();
    ctx.insert("language", "markdown");
    ctx.insert("code", "## Paste Your Code Here!");
    ctx.insert("pwd", "false");
    ctx.insert("notfound", "false");
    ctx.insert("fname", "");
    ctx.insert("author", "");
    ctx.insert("bgntime", "");
    ctx.insert("exptime", "");
    ctx.insert("desc", "");
    let rendered = data.templates.render("paste.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

async fn post_data(form: web::Form<PostData>, req: HttpRequest) -> impl Responder {
    let mut data = form.into_inner();
    println!("{:?}", data);
    let js = serde_json::to_string(&data).expect("Failed to serialize");
    println!("{}", js);
    let page_id = Uuid::new_v4().to_string();
    let file = File::create(format!("data/{}.json", page_id));
    match file {
        Ok(mut f) => {
            f.write_all(js.as_bytes());
        }
        Err(_) => {
            println!("Create File Error!!")
        }
    };
    let resp_json = json!({
        "url": format!("{}/paste/{}", req.connection_info().host(), page_id),
        "id": page_id,
    });
    HttpResponse::Ok().body(resp_json.to_string())
}

async fn error_page(data: web::Data<AppState>) -> impl Responder {
    let mut ctx = Context::new();
    let rendered = data.templates.render("paste.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

async fn get_page(info: web::Path<(String)>, state: web::Data<AppState>) -> impl Responder {
    let page_id = info.into_inner();
    println!("{}", page_id);
    match File::open(format!("data/{}.json", page_id)) {
        Ok(mut d) => {
            let mut json = String::new();
            d.read_to_string(&mut json);//todo handle the err
            let data: PostData = serde_json::from_str(&json).expect("File Broken!!");
            let mut ctx = Context::new();
            ctx.insert("language", &data.lang);
            //println!("{}", &data.code);
            ctx.insert("code", &data.code);
            ctx.insert("pwd", "false");
            ctx.insert("notfound", "false");
            ctx.insert("fname", &data.fname);
            ctx.insert("author", &data.user);
            ctx.insert("bgntime", "1");
            ctx.insert("exptime", &data.exp);
            ctx.insert("desc", &data.desc);
            let rendered = state.templates.render("paste.html", &ctx).unwrap();
            return HttpResponse::Ok().body(rendered);
        }
        Err(_) => {
            let mut ctx = Context::new();
            let rendered = state.templates.render("404.html", &ctx).unwrap();
            return HttpResponse::Ok().body(rendered);
        }
    }
}