use std::fs;

use actix_files::Files;
use actix_web::{
	cookie::{
		time::Duration,
		Cookie
	}, error::ErrorInternalServerError, web, HttpRequest, HttpResponse
};
use rusqlite::params;
use serde::{Deserialize, Serialize};

mod common;
mod scene;
mod battle;
mod func;
mod admin;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/strings")
		.route("", web::get().to(index))
		.route("/rulebook", web::get().to(rulebook))
		.route("/characters", web::get().to(characters))
		.route("/register", web::post().to(func::register))
		.route("/login", web::post().to(func::login))
		.route("/send_chat", web::post().to(func::send_chat))
		.route("/delete_chat", web::post().to(func::delete_chat))
		.route("/update_fragments", web::post().to(func::update_fragments))
		.route("/create_fragment", web::post().to(func::create_fragment))
		.route("/update_profile", web::post().to(func::update_profile))
		.route("/send_battle", web::post().to(func::send_battle))
		.route("/receive_battle", web::post().to(func::receive_battle))
		.route("/cancel_battle", web::post().to(func::cancel_battle))
		.route("/get_location", web::get().to(func::get_location))
		.route("/get_chat", web::get().to(func::get_chat))
		.route("/get_characters", web::get().to(func::get_characters))
		.route("/get_fragments", web::get().to(func::get_fragments))
		.route("/get_has_kins", web::get().to(func::get_has_kins))
		.route("/get_profile", web::get().to(func::get_profile))
		.route("/get_battle_log", web::get().to(func::get_battle_log))
		.route("/get_battle_logs", web::get().to(func::get_battle_logs))
		.route("/get_battle_reserve", web::get().to(func::get_battle_reserve))
		.route("/next", web::post().to(scene::next))
		.route("/admin", web::get().to(admin::index))
		.route("/admin/get_characters", web::get().to(admin::get_characters))
		.route("/admin/get_fragments", web::get().to(admin::get_fragments))
		.route("/admin/get_skills", web::get().to(admin::get_skills))
		.route("/admin/get_players_fragments", web::get().to(admin::get_players_fragments))
		.route("/admin/get_npcs", web::get().to(admin::get_npcs))
		.route("/admin/execute_sql", web::post().to(admin::execute_sql))
		.route("/admin/update_character", web::post().to(admin::update_character))
		.route("/admin/update_fragment", web::post().to(admin::update_fragment))
		.route("/admin/update_skill", web::post().to(admin::update_skill))
		.route("/admin/update_players_fragment", web::post().to(admin::update_players_fragment))
		.route("/admin/update_npc", web::post().to(admin::update_npc))
        .service(Files::new("/", "resource/strings").show_files_listing())
    );
}

#[derive(Serialize, Deserialize)]
struct FormFragment {
    name: String,
    lore: String,
}
// 最初にアクセスした際にログインセッションを確認し、ゲームや登録ページを表示する
async fn index(req: HttpRequest) -> Result<HttpResponse, actix_web::Error> {
    // データベースに接続
    let conn = common::open_database()?;
    // サーバーの状態を確認
    if let Err(state) = common::check_server_state(&conn)? {
        match state.as_str() {
            "maintenance" => return Ok(HttpResponse::Ok()
                .body(
                    liquid::ParserBuilder::with_stdlib()
                        .build().map_err(|err| ErrorInternalServerError(err))?
                        .parse(&fs::read_to_string("html/maintenance.html").unwrap()).map_err(|err| ErrorInternalServerError(err))?
                        .render(&liquid::object!({})).map_err(|err| ErrorInternalServerError(err))?
                )
            ),
            "end" => return Ok(HttpResponse::Ok()
                .body(
                    liquid::ParserBuilder::with_stdlib()
                        .build().map_err(|err| ErrorInternalServerError(err))?
                        .parse(&fs::read_to_string("html/entrance.html").unwrap()).map_err(|err| ErrorInternalServerError(err))?
                        .render(&liquid::object!({"fragment":Vec::<FormFragment>::new()})).map_err(|err| ErrorInternalServerError(err))?
                )
            ),
            _ => return Err(ErrorInternalServerError("サーバーが不明な状態です。"))
        }
    }
    // 期限切れのログインセッションを削除
    conn.execute("DELETE FROM login_session WHERE timestamp<datetime('now','-7 days')", [])
        .map_err(|err| ErrorInternalServerError(err))?;
    // Cookieに保存されているログインセッションを取得
    if let Some(session) = req.cookie("login_session") {
        // ログインセッションをデータベースと照会
        if let Ok(eno) = common::session_to_eno(&conn, session.value()) {
            // ログインセッションのタイムスタンプを更新
            conn.execute("UPDATE login_session SET timestamp=CURRENT_TIMESTAMP WHERE id=?1", params![session.value()])
                .map_err(|err| ErrorInternalServerError(err))?;
            // 各種情報取得
            let (name, color, location, display, webhook): (String, [u8; 3], String, String, Option<String>) = conn
                .query_row(
                    "SELECT name,color,location,display,webhook FROM character c INNER JOIN scene s ON c.eno=?1 AND c.eno=s.eno INNER JOIN user u ON c.eno=u.eno",
                    params![eno],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
                ).map_err(|err| ErrorInternalServerError(err))?;
            let display: Vec<&str> = if display != "" { display.split('\n').collect() } else { Vec::new() };
            let lore: String = conn.query_row("SELECT lore FROM location WHERE name=?1", params![location], |row| Ok(row.get(0)?))
                .unwrap_or("この場所の情報はない。".to_string());
            // 返却
            return || -> Result<HttpResponse, liquid::Error> {
                Ok(HttpResponse::Ok()
                    .body(
                        liquid::ParserBuilder::with_stdlib()
                            .build()?
                            .parse(&fs::read_to_string("html/game.html").unwrap())?
                            .render(&liquid::object!({"eno":eno, "name":name, "color":format!("#{:02x}{:02x}{:02x}", color[0], color[1], color[2]), "location":{"name":location, "lore":lore}, "display":display, "webhook":webhook.unwrap_or(String::new()) }))?
                    )
                )
            }().map_err(|err| ErrorInternalServerError(err));
        }
    }
    // どこかで終了したらエントランスを表示
    // 形質フラグメントのリストを取得
    let result = || -> Result<_, rusqlite::Error> {
        let mut stmt = conn.prepare("SELECT name,lore FROM form_fragment")?;
        let result = stmt.query_map([], |row| {
            Ok(FormFragment{ name: row.get(0)?, lore: row.get(1)? })
        })?.collect::<Result<Vec<_>, _>>();
        result
    }().map_err(|err| ErrorInternalServerError(err))?;
    // 表示
    || -> Result<HttpResponse, liquid::Error> {
        Ok(HttpResponse::Ok()
            .cookie(Cookie::build("login_session", "").max_age(Duration::ZERO).finish())
            .body(
                liquid::ParserBuilder::with_stdlib()
                    .build()?
                    .parse(&fs::read_to_string("html/entrance.html").unwrap())?
                    .render(&liquid::object!({"fragment":result }))?
            )
        )
    }().map_err(|err| ErrorInternalServerError(err))
}

async fn rulebook() -> HttpResponse {
    HttpResponse::Ok()
        .body(fs::read_to_string("html/rulebook.html").unwrap())
}
async fn characters() -> HttpResponse {
    HttpResponse::Ok()
        .body(fs::read_to_string("html/characters.html").unwrap())
}

#[allow(dead_code)]
pub fn test() {
    let _ = admin::preset();
    // let mut data = std::collections::HashMap::from([("select".to_string(), "throw".to_string())]);
    // match scene::process_line(fs::read_to_string("game/scene/test").unwrap().as_str(), 1, &mut data) {
    //     Ok(ok) => {
    //         println!("{}", ok);
    //     }
    //     Err(err) => {
    //         println!("{}", err);
    //     }
    // }
}