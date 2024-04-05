use std::{collections::hash_map::DefaultHasher, hash::{Hash, Hasher}, i128};

use actix_rt;
use actix_web::{error::{ErrorBadRequest, ErrorInternalServerError, ErrorServiceUnavailable}, web::{self, Json}, HttpRequest};
use rusqlite::{named_params, params, types::Type};
use serde::{Deserialize, Serialize};
use unicode_segmentation::UnicodeSegmentation;
use uuid::Uuid;

use super::{admin, battle, common, FormFragment};

// アプリケーション部分
#[derive(Deserialize)]
pub(super) struct LoginData {
    eno: i16,
    password: String,
}
pub(super) async fn login(info: web::Json<LoginData>) -> Result<String, actix_web::Error> {
    // データベースに接続
    let conn = common::open_database()?;
    // サーバーの状態を確認
    if let Err(state) = common::check_server_state(&conn)? {
        match state.as_str() {
            "maintenance" => return Err(ErrorServiceUnavailable("メンテナンス中につき操作できません")),
            "end" => return Err(ErrorServiceUnavailable("このサイトは稼働終了しました")),
            "littlegirl" => (),
            _ => return Err(ErrorInternalServerError(common::SERVER_UNDEFINED_TEXT))
        }
    }
    // postで受け取ったパスワードをハッシュ化する
    let mut hasher = DefaultHasher::new();
    info.password.hash(&mut hasher);
    // データベースを探索
    let eno = conn.query_row(
        "SELECT eno FROM user WHERE eno=?1 AND password=?2",
        params![info.eno, hasher.finish() as i64],
        |row| row.get::<usize, i16>(0),
    ).map_err(|err| match err {
        rusqlite::Error::QueryReturnedNoRows => ErrorBadRequest("Enoまたはパスワードが違います"),
        _ => ErrorBadRequest(err)
    })?;
    // ログインセッションを生成
    let login_session = Uuid::new_v4().as_u128() as i128;
    // println!("login: {},{}", login_session, eno);
    // ログインセッションとEnoをデータベースに保存
    conn.execute("INSERT INTO login_session(id,eno) VALUES(?1,?2)", params![login_session, eno])
        .map_err(|err| ErrorInternalServerError(err))?;
    // ログインセッションを文字列形式で返す
    Ok(login_session.to_string())
}

#[derive(Deserialize)]
pub(super) struct RegisterData {
    password: String,
    name: String,
    acronym: String,
    color: String,
    fragment: Vec<FormFragment>,
}
pub(super) async fn register(info: web::Json<RegisterData>) -> Result<String, actix_web::Error> {
    // 与えられた情報の整合性確認
    let password_length = info.password.chars().count();
    if password_length > 16 || password_length < 4 {
        return Err(ErrorBadRequest("パスワードは4文字以上16文字以下に設定してください"));
    } else if info.name.grapheme_indices(true).count() > 16 {
        return Err(ErrorBadRequest("名前が長すぎます"));
    } else if info.acronym.grapheme_indices(true).count() != 1 {
        return Err(ErrorBadRequest("短縮名が1文字ではありません"));
    } else if info.fragment.len() > 5 {
        return Err(ErrorBadRequest("取得しようとしているフラグメントが多すぎます"));
    }
    // println!("{}", &info.color[1..]);
    let color_raw = u32::from_str_radix(&info.color[1..], 16).map_err(|err| ErrorBadRequest(err))?;
    let color = [(color_raw >> 16) as u8, (color_raw >> 8) as u8, color_raw as u8];
    // データベースに接続
    let conn = common::open_database()?;
    // サーバーの状態を確認
    if let Err(state) = common::check_server_state(&conn)? {
        match state.as_str() {
            "maintenance" => return Err(ErrorServiceUnavailable("メンテナンス中につき操作できません")),
            "end" => return Err(ErrorServiceUnavailable("このサイトは稼働終了しました")),
            "littlegirl" => (),
            _ => return Err(ErrorInternalServerError(common::SERVER_UNDEFINED_TEXT))
        }
    }
    // 発生するエラーはすべてInternalServerError相当なので、クロージャに格納してまとめてmapしている
    || -> Result<_, rusqlite::Error> {
        // 受け取ったパスワードをハッシュ化
        let mut hasher = DefaultHasher::new();
        info.password.hash(&mut hasher);
        // データベースに新規ユーザーを追加
        conn.execute("INSERT INTO user(password) VALUES(?1)", params![hasher.finish() as i64])?;
        let eno = conn.last_insert_rowid() as i16;
        conn.execute("INSERT INTO character VALUES(?1,?2,?3,?4,?5,?6,?7,?8,true)", params![eno, info.name, info.acronym, color, "", "門", 0, "{}"])?;
        conn.execute("INSERT INTO character_profile VALUES(?1,'','')", params![eno])?;
        conn.execute("INSERT INTO scene VALUES(?1,'','','{}','')", params![eno])?;
        // フラグメント追加
        let mut sql = "INSERT INTO fragment(eno,slot,category,name,lore,status) VALUES".to_string();
        let mut params = params![eno, [0u8; 8]].to_vec();
        let mut i = 1;
        for f in &info.fragment {
            i += 1;
            sql += &format!("{}(?1,{},'形質',?{},?{},?2)", if i > 2 { "," } else { "" }, i - 1, i * 2 - 1, i * 2);
            params.push(&f.name);
            params.push(&f.lore);
        }
        conn.execute(&sql, params.as_slice())?;
        // 登録成功したらログインセッションを生成
        let login_session = Uuid::new_v4().as_u128() as i128;
        conn.execute("INSERT INTO login_session(id,eno) VALUES(?1,?2)", params![login_session, eno])?;
        Ok(login_session.to_string())
    }().map_err(|err| ErrorInternalServerError(err))
}

#[derive(Deserialize)]
pub(super) struct SendChatData {
    name: String,
    word: String,
    location: bool,
    to: Option<i16>,
}
pub(super) async fn send_chat(req: HttpRequest, info: web::Json<SendChatData>) -> Result<String, actix_web::Error> {
    // ログインセッションを取得
    let session =  req.cookie("login_session")
        .ok_or(ErrorBadRequest("ログインセッションがありません"))?;
    // 発言内容をエスケープ
    let mut word = common::html_special_chars(&info.word);
    // 入力情報が正しい長さであることを確認
    if info.name.graphemes(true).count() <= 20 && word.graphemes(true).count() <= 600 {
        let handle = if let Some(to) = &info.to {
            Some(actix_rt::spawn(common::send_webhook(*to, format!("Eno.{} {}からの発言を受けました。", to, info.name))))
        } else { None };
        // データベースに接続
		let conn = common::open_database()?;
        // サーバーの状態を確認
        if let Err(state) = common::check_server_state(&conn)? {
            match state.as_str() {
                "maintenance" => return Err(ErrorServiceUnavailable(common::SERVER_MAINTENANCE_TEXT)),
                "end" => return Err(ErrorServiceUnavailable(common::SERVER_END_TEXT)),
                "littlegirl" => return Err(ErrorServiceUnavailable(common::SERVER_LITTLEGIRL_TEXT)),
                _ => return Err(ErrorInternalServerError(common::SERVER_UNDEFINED_TEXT))
            }
        }
        // ログインセッションをデータベースと照会
        let (eno, visit) = common::session_to_eno(&conn, session.value())?;
        if !visit {
            return Err(ErrorBadRequest("あなたは現在\"Strings\"内にいません"));
        }
        // 短縮名・発言色・現在地を取得
        let (acronym, color, location): (String, [u8; 3], String) = conn.query_row(
            "SELECT acronym,color,location FROM character WHERE eno=?1",
            params![eno],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        ).map_err(|err| ErrorInternalServerError(err))?;
        // タグを置換
        word = common::replace_tag(&word, eno, false).map_err(|err| ErrorInternalServerError(err))?;
        // 発言をデータベースに格納
        conn.execute(
            "INSERT INTO timeline(from_eno,to_eno,location,acronym,color,name,word) VALUES(?1,?2,?3,?4,?5,?6,?7)",
            params![eno, info.to, if info.location { None } else { Some(location) }, acronym, color, info.name, word],
        ).map_err(|err| ErrorInternalServerError(err))?;
        if let Some(h) = handle {
            let _ = h.await;
        }
        Ok("発言に成功しました".to_string())
    } else {
        Err(ErrorBadRequest("名前、または発言内容が長すぎます"))
    }
}

#[derive(Deserialize)]
pub(super) struct DeleteChatData {
    id: i32,
}
pub(super) async fn delete_chat(req: HttpRequest, info: web::Json<DeleteChatData>) -> Result<String, actix_web::Error> {
    // ログインセッションを取得
    let session =  req.cookie("login_session")
        .ok_or(ErrorBadRequest("ログインセッションがありません"))?;
    // データベースに接続
    let conn = common::open_database()?;
    // サーバーの状態を確認
    if let Err(state) = common::check_server_state(&conn)? {
        match state.as_str() {
            "maintenance" => return Err(ErrorServiceUnavailable(common::SERVER_MAINTENANCE_TEXT)),
            "end" => return Err(ErrorServiceUnavailable(common::SERVER_END_TEXT)),
            "littlegirl" => (),
            _ => return Err(ErrorInternalServerError(common::SERVER_UNDEFINED_TEXT))
        }
    }
    // Enoを取得
    let (eno, visit) = common::session_to_eno(&conn, session.value())?;
    if !visit {
        return Err(ErrorBadRequest("あなたは現在\"Strings\"内にいません"));
    }
    // 削除できなかった時にキャッチする方法が特にない
    conn.execute("UPDATE timeline SET live=false WHERE id=?1 AND from_eno=?2", params![info.id, eno])
        .map_err(|err| ErrorInternalServerError(err))?;
    Ok(String::new())
}

#[derive(Deserialize)]
pub(super) struct GetChatData {
    num: i32,                   // 取得件数
    start: Option<i32>,         // 取得開始位置
    from: Option<String>,       // 発言者
    to: Option<String>,         // 対象者
    location: Option<String>,   // 取得座標
    word: Option<String>,       // 検索文字列
}
#[derive(Serialize)]
pub(super) struct Chat {
    id: i32,
    timestamp: String,
    from: i16,
    to: Option<i16>,
    location: Option<String>,
    color: [u8; 3],
    acronym: String,
    name: String,
    word: String,
}
pub(super) async fn get_chat(req: HttpRequest, info: web::Query<GetChatData>) -> Result<web::Json<Vec<Chat>>, actix_web::Error> {
    // データベースに接続
    let conn = common::open_database()?;
    // サーバーの状態を確認
    if let Err(state) = common::check_server_state(&conn)? {
        match state.as_str() {
            "maintenance" => {
                // パスワード取得
                let password =  req.cookie("admin_password")
                    .ok_or(ErrorServiceUnavailable(common::SERVER_MAINTENANCE_TEXT))?;
                // パスワード確認
                admin::check_server_password(&conn, password.value())?;
            },
            "end" => (),
            "littlegirl" => {
                // データベースから取得
                return || -> Result<_, rusqlite::Error> {
                    let mut stmt = conn.prepare("SELECT id,datetime(timestamp,'+9 hours'),from_eno,to_eno,location,acronym,color,name,word FROM timeline WHERE live=true AND from_eno=0 ORDER BY id DESC LIMIT ?1")?;
                    let result = stmt.query_map(
                        params![info.num],
                        |row| {
                            Ok(Chat {
                                id: row.get(0)?,
                                timestamp: row.get(1)?,
                                from: row.get(2)?,
                                to: row.get(3)?,
                                location: row.get(4)?,
                                acronym: row.get(5)?,
                                color: row.get(6)?,
                                name: row.get(7)?,
                                word: row.get(8)?,
                            })
                        }
                    )?.collect::<Result<Vec<_>, _>>()?;
                    Ok(web::Json(result))
                }().map_err(|err| ErrorInternalServerError(err));
            },
            _ => return Err(ErrorInternalServerError(common::SERVER_UNDEFINED_TEXT))
        }
    }
    // 自身のEnoを（あるなら）取得
    let eno = if let Some(session) = req.cookie("login_session") {
        common::session_to_eno(&conn, session.value()).ok()
    } else { None };
    // 発言者・対象者条件を生成
    let (plus, minus) = if info.from != None || info.to != None {
        // 発言者リストの条件文生成
        let mut plus = Vec::new();
        let mut minus = Vec::new();
        if let Some(v) = &info.from {
            // 数値リストを取得
            let v = v.replace(" ", "").split(',').map(|x| {
                match x.parse::<i16>() {
                    Ok(v) => {
                        if v == 0 {
                            match eno {
                                Some((eno, _)) => {
                                    if x.chars().nth(0) == Some('-') {
                                        Ok(-eno)
                                    } else {
                                        Ok(eno)
                                    }
                                },
                                None => Err(ErrorBadRequest("Eno条件に0を使用する場合はログインセッションが必要です"))
                            }
                        } else {
                            Ok(v)
                        }
                    }
                    Err(_) => Err(ErrorBadRequest("Eno条件に数値でないものが含まれています"))
                }
            }).collect::<Result<Vec<_>, _>>()?;
            // 検索対象
            let p = v.iter().filter_map(|x| {
                if *x > 0 {
                    Some(x.to_string())
                } else { None }
            }).collect::<Vec<_>>();
            // 除外対象
            let m = v.iter().filter_map(|x| {
                if *x < 0 {
                    Some(x.abs().to_string())
                } else { None }
            }).collect::<Vec<_>>();
            // 返却
            if p.len() != 0 {
                plus.push(format!("from_eno IN ({})", p.join(",")));
            }
            if m.len() != 0 {
                minus.push(format!("from_eno NOT IN ({})", m.join(",")));
            }
        }
        // 対象者リストの書式チェック
        if let Some(v) = &info.to {
            // 数値リストを取得
            let v = v.replace(" ", "").split(',').map(|x| {
                match x.parse::<i16>() {
                    Ok(v) => {
                        if v == 0 {
                            match eno {
                                Some((eno, _)) => {
                                    if x.chars().nth(0) == Some('-') {
                                        Ok(-eno)
                                    } else {
                                        Ok(eno)
                                    }
                                },
                                None => Err(ErrorBadRequest("Eno条件に0を使用する場合はログインセッションが必要です"))
                            }
                        } else {
                            Ok(v)
                        }
                    }
                    Err(_) => Err(ErrorBadRequest("Eno条件に数値でないものが含まれています"))
                }
            }).collect::<Result<Vec<_>, _>>()?;
            // 検索対象
            let p = v.iter().filter_map(|x| {
                if *x > 0 {
                    Some(x.to_string())
                } else { None }
            }).collect::<Vec<_>>();
            // 除外対象
            let m = v.iter().filter_map(|x| {
                if *x < 0 {
                    Some(x.abs().to_string())
                } else { None }
            }).collect::<Vec<_>>();
            // 返却
            if p.len() != 0 {
                plus.push(format!("to_eno IN ({})", p.join(",")));
            }
            if m.len() != 0 {
                minus.push(format!("to_eno NOT IN ({})", m.join(",")));
            }
        }
        // (from IN (...) OR to IN (...)) AND (from NOT IN (...) OR to NOT IN (...))
        (if plus.len() != 0 { Some(format!("({})", plus.join(" OR "))) } else { None }, if minus.len() != 0 { Some(format!("({})", minus.join(" OR "))) } else { None })
    } else { (None, None) };
    // ロケーション指定
    let location = match &info.location {
        // ロケーションが指定されている
        Some(location) => {
            // 文字数が64Byte以下なのを確認（SQL検索用なので文字数ではなくByte制限）
            if location.len() < 64 {
                if location == "*" {
                    // 全件検索
                    None
                } else {
                    // 単一指定
                    Some(location.clone())
                }
            } else {
                return Err(ErrorBadRequest("ロケーション名が長すぎます"));
            }
        }
        // ロケーションが指定されていない（現在地参照）
        None => {
            if let Some((eno, _)) = eno {
                Some(conn.query_row("SELECT location FROM character WHERE eno=?1", params![eno], |row|Ok(row.get(0)?))
                    .map_err(|err|ErrorInternalServerError(err))?)
            } else {
                return Err(ErrorBadRequest("現在位置を参照する検索にはログインセッションが必要です"));
            }
        }
    };
    // SQL文とパラメータを生成
    let mut sql = Vec::new();
    let mut params = named_params![":num":info.num].to_vec();
    // キャラクター
    if let Some(plus) = &plus {
        sql.push(plus.as_str());
    }
    if let Some(minus) = &minus {
        sql.push(minus.as_str());
    }
    // ロケーション
    if let Some(location) = &location {
        sql.push("location=:location");
        params.push((":location", location));
    }
    // 開始位置
    if let Some(start) = &info.start {
        sql.push("id<:start");
        params.push((":start", start));
    }
    // 検索文字列
    if let Some(word) = &info.word {
        if word.graphemes(true).count() < 32 {
            sql.push("word LIKE '%'||:word||'%'");
            params.push((":word", word));
        } else {
            return Err(ErrorBadRequest("検索する文字列が長すぎます"));
        }
    }
    let sql = "SELECT id,datetime(timestamp,'+9 hours'),from_eno,to_eno,location,acronym,color,name,word FROM timeline WHERE live=true".to_string() + if sql.is_empty() { "" } else { " AND " } + &sql.join(" AND ") + " ORDER BY id DESC LIMIT :num";
    // データベースから取得
    || -> Result<_, rusqlite::Error> {
        let mut stmt = conn.prepare(sql.as_str())?;
        let result = stmt.query_map(
            params.as_slice(),
            |row| {
                Ok(Chat {
                    id: row.get(0)?,
                    timestamp: row.get(1)?,
                    from: row.get(2)?,
                    to: row.get(3)?,
                    location: row.get(4)?,
                    acronym: row.get(5)?,
                    color: row.get(6)?,
                    name: row.get(7)?,
                    word: row.get(8)?,
                })
            }
        )?.collect::<Result<Vec<_>, _>>()?;
        Ok(web::Json(result))
    }().map_err(|err| ErrorInternalServerError(err))
}

#[derive(Deserialize)]
pub(super) struct GetCharacterData {
    num: i32,
    start: Option<i32>,
    location: Option<String>,
}
#[derive(Serialize)]
pub(super) struct Character {
    eno: i16,
    name: String,
    acronym: String,
    color: [u8; 3],
    comment: String,
    location: String,
}
pub(super) async fn get_characters(req: HttpRequest, info: web::Query<GetCharacterData>) -> Result<web::Json<Vec<Character>>, actix_web::Error> {
    // データベースに接続
    let conn = common::open_database()?;
    // サーバーの状態を確認
    if let Err(state) = common::check_server_state(&conn)? {
        match state.as_str() {
            "maintenance" => return Err(ErrorServiceUnavailable(common::SERVER_MAINTENANCE_TEXT)),
            "end" => (),
            "littlegirl" => (),
            _ => return Err(ErrorInternalServerError(common::SERVER_UNDEFINED_TEXT))
        }
    }
    // 自身のEnoを（あるなら）取得
    let eno = if let Some(session) = req.cookie("login_session") {
        common::session_to_eno(&conn, session.value()).ok()
    } else { None };
    // ロケーション指定
    let location = match &info.location {
        // ロケーションが指定されている
        Some(location) => {
            // 文字数が64Byte以下なのを確認（SQL検索用なので文字数ではなくByte制限）
            if location.len() < 64 {
                if location == "*" {
                    // 全件検索
                    None
                } else {
                    // 単一指定
                    Some(location.clone())
                }
            } else {
                return Err(ErrorBadRequest("ロケーション名が長すぎます"));
            }
        }
        // ロケーションが指定されていない（現在地参照）
        None => {
            if let Some((eno, _)) = eno {
                Some(conn.query_row("SELECT location FROM character WHERE eno=?1", params![eno], |row|Ok(row.get(0)?))
                    .map_err(|err|ErrorInternalServerError(err))?)
            } else {
                return Err(ErrorBadRequest("現在位置を参照する検索にはログインセッションが必要です"));
            }
        }
    };
    // SQL文とパラメータを生成
    let mut sql = Vec::new();
    let mut params = named_params![":num":info.num].to_vec();
    // ロケーション
    if let Some(location) = &location {
        sql.push("location=:location");
        params.push((":location", location));
    }
    // 開始位置
    if let Some(start) = &info.start {
        sql.push("eno>:start");
        params.push((":start", start));
    }
    let sql = sql.join(" AND ");
    let sql = "SELECT eno,name,acronym,color,comment,location FROM character WHERE visit=true".to_string() + if sql != "" { " AND " } else { "" } + &sql + " ORDER BY eno ASC LIMIT :num";
    // データベースから取得
    || -> Result<_, rusqlite::Error> {
        let mut stmt = conn.prepare(sql.as_str())?;
        let characters = stmt.query_map(
            params.as_slice(),
            |row| {
                Ok(Character {
                    eno: row.get(0)?,
                    name: row.get(1)?,
                    acronym: row.get(2)?,
                    color: row.get(3)?,
                    comment: row.get(4)?,
                    location: row.get(5)?,
                })
            }
        )?.collect::<Result<Vec<_>, _>>()?;
        Ok(web::Json(characters))
    }().map_err(|err| ErrorInternalServerError(err))
}

#[derive(Serialize)]
struct Skill {
    id: i32,
    name: Option<String>,
    word: Option<String>,
    default_name: String,
    lore: String,
    timing: battle::Timing,
    effect: battle::Effect,
}
#[derive(Serialize)]
pub(super) struct Fragment {
    slot: i8,
    category: String,
    name: String,
    lore: String,
    hp: i16,
    mp: i16,
    atk: i16,
    tec: i16,
    skill: Option<Skill>,
    user: bool,
}
pub(super) async fn get_fragments(req: HttpRequest) -> Result<web::Json<Vec<Fragment>>, actix_web::Error> {
    // ログインセッションを取得
    let session =  req.cookie("login_session")
        .ok_or(ErrorBadRequest("ログインセッションがありません"))?;
    // データベースに接続
    let conn = common::open_database()?;
    // サーバーの状態を確認
    if let Err(state) = common::check_server_state(&conn)? {
        match state.as_str() {
            "maintenance" => return Err(ErrorServiceUnavailable(common::SERVER_MAINTENANCE_TEXT)),
            "end" => (),
            "littlegirl" => (),
            _ => return Err(ErrorInternalServerError(common::SERVER_UNDEFINED_TEXT))
        }
    }
    // Enoを取得
    let (eno, _) = common::session_to_eno(&conn, session.value())?;
    // 取得・整形
    || -> Result<_, rusqlite::Error> {
        let mut stmt = conn.prepare("WITH f AS (SELECT slot,category,name,lore,status,skill,skillname,skillword,user FROM fragment WHERE eno=?1) SELECT f.*,s.name,s.lore,s.type,s.effect FROM f LEFT OUTER JOIN skill s ON f.skill=s.id")?;
        let result = stmt.query_map(params![eno], |row| {
            // ステータスの原型
            let status: Vec<u8> = row.get(4)?;
            // スキルの有無を判定・整形
            let skill = if let (Some(id), Some(name), Some(lore), Some(timing), Some(effect)) = (row.get(5)?, row.get(9)?, row.get(10)?, row.get::<_, Option<i8>>(11)?, row.get(12)?) {
                let timing = timing.into();
                let effect = if timing == battle::Timing::World {
                    battle::Effect::World(battle::WorldEffect::convert(effect).map_err(|_| rusqlite::Error::InvalidColumnType(12, "skill.effect".to_string(), Type::Blob))?)
                } else {
                    battle::Effect::Formula(battle::Command::convert(effect).map_err(|_| rusqlite::Error::InvalidColumnType(12, "skill.effect".to_string(), Type::Blob))?)
                };
                Some(Skill {
                    id,
                    name: row.get(6)?,
                    word: row.get(7)?,
                    default_name: name,
                    lore,
                    timing,
                    effect,
                })
            } else { None };
            // 取得返却
            Ok(Fragment {
                slot: row.get(0)?,
                category: row.get(1)?,
                name: row.get(2)?,
                lore: row.get(3)?,
                hp: (status[0] as i16) << 8 | status[1] as i16,
                mp: (status[2] as i16) << 8 | status[3] as i16,
                atk: (status[4] as i16) << 8 | status[5] as i16,
                tec: (status[6] as i16) << 8 | status[7] as i16,
                skill,
                user: row.get(8)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        Ok(web::Json(result))
    }().map_err(|err| ErrorInternalServerError(err))
}

#[derive(Deserialize)]
struct ChangeFragmentData {
    prev: i8,
    next: i8,
    skill_name: Option<String>,
    skill_word: Option<String>,
    trash: Option<bool>,
    pass: Option<i16>,
}
#[derive(Deserialize)]
pub(super) struct UpdateFragmentsData {
    change: Vec<ChangeFragmentData>,
}
struct UpdateFragment {
    slot: i8,
    category: String,
    name: String,
    lore: String,
    status: [u8; 8],
    skill: Option<i32>,
    user: bool,
}
struct PartFragmentSkill {
    skill_name: Option<String>,
    skill_word: Option<String>,
}
#[derive(Serialize)]
pub(super) struct ResultUpdateFragments {
    pass_error: Vec<String>,
    update_error: Vec<String>,
    trash_error: Vec<String>,
}
pub(super) async fn update_fragments(req: HttpRequest, info: web::Json<UpdateFragmentsData>) -> Result<Json<ResultUpdateFragments>, actix_web::Error> {
    // ログインセッションを取得
    let session =  req.cookie("login_session")
        .ok_or(ErrorBadRequest("ログインセッションがありません"))?;
    // データベースに接続
    let conn = common::open_database()?;
    // サーバーの状態を確認
    if let Err(state) = common::check_server_state(&conn)? {
        match state.as_str() {
            "maintenance" => return Err(ErrorServiceUnavailable(common::SERVER_MAINTENANCE_TEXT)),
            "end" => return Err(ErrorServiceUnavailable(common::SERVER_END_TEXT)),
            "littlegirl" => (),
            _ => return Err(ErrorInternalServerError(common::SERVER_UNDEFINED_TEXT))
        }
    }
    // Enoを取得
    let (eno, visit) = common::session_to_eno(&conn, session.value())?;
    if !visit {
        return Err(ErrorBadRequest("あなたは現在\"Strings\"内にいません"));
    }
    // フラグメントの一覧を取得
    let result = || -> Result<_, rusqlite::Error> {
        let mut stmt = conn.prepare("SELECT slot,category,name,lore,status,skill,user FROM fragment WHERE eno=?1")?;
        let result = stmt.query_map(params![eno], |row| {
            Ok(UpdateFragment {
                slot: row.get(0)?,
                category: row.get(1)?,
                name: row.get(2)?,
                lore: row.get(3)?,
                status: row.get(4)?,
                skill: row.get(5)?,
                user: row.get(6)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        Ok(result)
    }().map_err(|err| ErrorInternalServerError(err))?;
    // エラーフラグメントリスト
    let mut result_update_fragments = ResultUpdateFragments { pass_error: Vec::new(), update_error: Vec::new(), trash_error: Vec::new() };
    // 削除リスト作成
    let mut delete_list = Vec::new();
    let mut kins = 0;
    for i in &info.change {
        // 破棄する場合
        if i.trash == Some(true) {
            // 変更元を取得
            if let Some(f) = result.iter().find(|x| x.slot == i.prev) {
                if f.category == "世界観" {
                    result_update_fragments.trash_error.push(f.name.clone());
                    continue;
                }
                // 獲得キンス計算
                kins += common::calc_fragment_kins(f.status);
            }
        }
        // 削除リストに追加
        delete_list.push(i.prev.to_string());
    }
    // 確実に数値なので直接SQL文に挿入
    let sql = format!("DELETE FROM fragment WHERE eno=?1 AND slot IN ({})", delete_list.join(","));
    // 削除実行
    conn.execute(&sql, params![eno]).map_err(|err| ErrorInternalServerError(err))?;
    // 変更
    let mut append_fragment = Vec::new();
    for i in &info.change {
        // 破棄対象でない場合
        if i.trash != Some(true) {
            // 変更元を取得
            if let Some(f) = result.iter().find(|x| x.slot == i.prev) {
                // 移動先フラグメントを取得（本来削除済みなのでないはず、あるとしたら更新順序）
                match conn.query_row("SELECT slot,category,name,lore,status,skill,skillname,skillword,user FROM fragment WHERE eno=?1 AND slot=?2", params![eno, i.next], |row| {
                    Ok((UpdateFragment {
                        slot: row.get(0)?,
                        category: row.get(1)?,
                        name: row.get(2)?,
                        lore: row.get(3)?,
                        status: row.get(4)?,
                        skill: row.get(5)?,
                        user: row.get(8)?,
                    }, PartFragmentSkill {
                        skill_name: row.get(6)?,
                        skill_word: row.get(7)?,
                    }))
                }) {
                    Ok(next) => {
                        // 取得できてしまったら退避させて削除
                        conn.execute("DELETE FROM fragment WHERE eno=?1 AND slot=?2", params![eno, next.0.slot])
                            .map_err(|err| ErrorInternalServerError(err))?;
                        append_fragment.push(next);
                    }
                    Err(rusqlite::Error::QueryReturnedNoRows) => (),
                    Err(err) => return Err(ErrorInternalServerError(err)),
                }
                let mut eno = eno;
                let mut slot = i.next;
                // passが指定されている場合
                if let Some(pass) = i.pass {
                    if f.category == "世界観" {
                        result_update_fragments.pass_error.push(f.name.clone());
                    } else {
                        // 取得者のスロットの空き状況を取得
                        match common::get_empty_slot(&conn, pass) {
                            // 空きがある
                            Ok(Some(s)) => {
                                // 対象とスロットをそちらに変更
                                eno = pass;
                                slot = s;
                            }
                            // 空きがない、または対象が存在しない
                            Ok(None) | Err(rusqlite::Error::QueryReturnedNoRows) => result_update_fragments.pass_error.push(f.name.clone()),
                            // エラー
                            Err(err) => return Err(ErrorInternalServerError(err)),
                        }
                    }
                }
                // 新しいフラグメントを追加
                conn.execute("INSERT INTO fragment VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
                    params![
                        eno,
                        slot,
                        f.category,
                        f.name,
                        f.lore,
                        f.status,
                        f.skill,
                        i.skill_name,
                        i.skill_word,
                        f.user,
                    ]
                ).map_err(|err| ErrorInternalServerError(err))?;
            }
        }
    }
    // 退避させたフラグメントを追加しなおし
    for i in append_fragment {
        // スロットの空きを取得
        if let Some(slot) = common::get_empty_slot(&conn, eno)
            .map_err(|err| ErrorInternalServerError(err))? {
            // 空きに追加
            conn.execute("INSERT INTO fragment VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
                params![
                    eno,
                    slot,
                    i.0.category,
                    i.0.name,
                    i.0.lore,
                    i.0.status,
                    i.0.skill,
                    i.1.skill_name,
                    i.1.skill_word,
                    i.0.user,
                ]
            ).map_err(|err| ErrorInternalServerError(err))?;
        } else {
            // 無かったら獲得キンスに計上して名前を保存
            kins += common::calc_fragment_kins(i.0.status);
            result_update_fragments.update_error.push(i.0.name);
        }
    }
    // キンス反映
    if kins != 0 {
        conn.execute("UPDATE character SET kins=kins+?1 WHERE eno=?2", params![kins, eno])
            .map_err(|err| ErrorInternalServerError(err))?;
    }
    Ok(web::Json(result_update_fragments))
}

#[derive(Deserialize)]
pub(super) struct CreateFragmentData {
    material: Vec<i8>,
    category: String,
    name: String,
    lore: String,
    force: Option<bool>,
}
pub(super) async fn create_fragment(req: HttpRequest, info: web::Json<CreateFragmentData>) -> Result<Json<i32>, actix_web::Error> {
    if info.force != Some(true) {
        // カテゴリ制限
        match info.category.as_str() {
            "名前" | 
            "世界観" |
            "秘匿" | 
            "身代わり" => return Err(ErrorBadRequest(format!("{}カテゴリのフラグメントを作成することはできません", info.category))),
            _ => (),
        }
    }
    // マテリアル個数制限
    if info.material.len() == 0 {
        return Err(ErrorBadRequest("一つ以上のフラグメントを素材にする必要があります"));
    }
    // 文字数制限
    if !(1..=8).contains(&info.category.grapheme_indices(true).count()) {
        return Err(ErrorBadRequest("カテゴリは1文字以上8文字以下に設定してください"));
    }
    if !(1..=16).contains(&info.name.grapheme_indices(true).count()) {
        return Err(ErrorBadRequest("フラグメント名は1文字以上16文字以下に設定してください"));
    }
    if info.lore.grapheme_indices(true).count() > 200 {
        return Err(ErrorBadRequest("説明文は200文字以内に設定してください"));
    }
    // ログインセッションを取得
    let session =  req.cookie("login_session")
        .ok_or(ErrorBadRequest("ログインセッションがありません"))?;
    // データベースに接続
    let conn = common::open_database()?;
    // サーバーの状態を確認
    if let Err(state) = common::check_server_state(&conn)? {
        match state.as_str() {
            "maintenance" => return Err(ErrorServiceUnavailable(common::SERVER_MAINTENANCE_TEXT)),
            "end" => return Err(ErrorServiceUnavailable(common::SERVER_END_TEXT)),
            "littlegirl" => (),
            _ => return Err(ErrorInternalServerError(common::SERVER_UNDEFINED_TEXT))
        }
    }
    // Enoを取得
    let (eno, visit) = common::session_to_eno(&conn, session.value())?;
    if !visit {
        return Err(ErrorBadRequest("あなたは現在\"Strings\"内にいません"));
    }
    // 説明文整形
    let lore = common::replace_tag(&common::html_special_chars(&info.lore), eno, false)
        .map_err(|err| ErrorInternalServerError(err))?;
    // フラグメントの一覧を取得
    let result = || -> Result<_, rusqlite::Error> {
        let mut stmt = conn.prepare("SELECT slot,category FROM fragment WHERE eno=?1")?;
        let result: Vec<(i8, String)> = stmt.query_map(params![eno], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
            ))
        })?.collect::<Result<_, _>>()?;
        Ok(result)
    }().map_err(|err| ErrorInternalServerError(err))?;
    let mut cost = 70;
    // カテゴリ所持判定
    if info.force != Some(true) {
        let mut has_category = false;
        for i in &info.material {
            if let Some(f) = result.iter().find(|&x| &x.0 == i) {
                if f.1 == "世界観" {
                    return Err(ErrorBadRequest("世界観カテゴリのフラグメントは合成素材にできません"));
                }
                // カテゴリ所持
                if info.category == f.1 {
                    has_category = true;
                }
            } else {
                return Err(ErrorBadRequest("所持していないフラグメントを素材にしようとしました"));
            }
        }
        if has_category {
            cost -= 20;
        }
    }
    // 名前一文字につき2
    cost += info.name.grapheme_indices(true).count() as i32 * 2;
    // 説明文1行（改行または30文字まで）ごとに8
    for i in info.lore.lines() {
        cost += ((i.grapheme_indices(true).count() as i32 - 1) / 30 + 1) * 8;
    }
    // 素材の数*10だけ割引
    cost -= info.material.len() as i32 * 10;
    // 最低保証
    cost = cost.max(10);
    // キンス取得
    let kins: i32 = conn.query_row("SELECT kins FROM character WHERE eno=?1", params![eno], |row| Ok(row.get(0)?))
        .map_err(|err| ErrorInternalServerError(err))?;
    // 支払えるかを確認
    if cost > kins {
        return Err(ErrorBadRequest("キンスを支払えません"));
    }
    let base = info.material[0];
    let other = if info.material.len() > 1 {&info.material[1..]} else {&[]};
    // フラグメント編集
    conn.execute("UPDATE fragment SET category=?1,name=?2,lore=?3,user=true WHERE eno=?4 AND slot=?5",
        params![
            info.category,
            info.name,
            lore,
            eno,
            base,
        ]
    ).map_err(|err| ErrorInternalServerError(err))?;
    // コスト減算
    conn.execute("UPDATE character SET kins=?1 WHERE eno=?2", params![kins - cost, eno])
        .map_err(|err| ErrorInternalServerError(err))?;
    // 素材フラグメント削除
    conn.execute(
        &format!("DELETE FROM fragment WHERE eno=?1 AND slot IN ({})", other.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")),
        params![eno],
    ).map_err(|err| ErrorInternalServerError(err))?;
    Ok(web::Json(cost))
}
pub(super) async fn get_has_kins(req: HttpRequest) -> Result<web::Json<i32>, actix_web::Error> {
    // ログインセッションを取得
    let session =  req.cookie("login_session")
        .ok_or(ErrorBadRequest("ログインセッションがありません"))?;
    // データベースに接続
    let conn = common::open_database()?;
    // サーバーの状態を確認
    if let Err(state) = common::check_server_state(&conn)? {
        match state.as_str() {
            "maintenance" => return Err(ErrorServiceUnavailable(common::SERVER_MAINTENANCE_TEXT)),
            "end" => (),
            "littlegirl" => (),
            _ => return Err(ErrorInternalServerError(common::SERVER_UNDEFINED_TEXT))
        }
    }
    // Enoを取得
    let (eno, _) = common::session_to_eno(&conn, session.value())?;
    // キンス取得
    let kins: i32 = conn.query_row("SELECT kins FROM character WHERE eno=?1", params![eno], |row| Ok(row.get(0)?))
        .map_err(|err| ErrorInternalServerError(err))?;
    Ok(web::Json(kins))
}

#[derive(Deserialize)]
pub(super) struct GetProfileData {
    eno: i16,
}
#[derive(Serialize, Clone)]
struct ProfileFragment {
    name: String,
    category: String,
    lore: String,
}
#[derive(Serialize)]
pub(super) struct Profile {
    eno: i16,
    acronym: String,
    color: [u8; 3],
    comment: String,
    fullname: String,
    profile: String,
    memo: String,
    fragments: Vec<ProfileFragment>,
    edit_mode: bool,
    name: Option<String>,
    raw_profile: Option<String>,
    raw_memo: Option<String>,
}
pub(super) async fn get_profile(req: HttpRequest, info: web::Query<GetProfileData>) -> Result<web::Json<Profile>, actix_web::Error> {
    // データベースに接続
    let conn = common::open_database()?;
    // サーバーの状態を確認
    if let Err(state) = common::check_server_state(&conn)? {
        match state.as_str() {
            "maintenance" => return Err(ErrorServiceUnavailable(common::SERVER_MAINTENANCE_TEXT)),
            "end" => (),
            "littlegirl" => (),
            _ => return Err(ErrorInternalServerError(common::SERVER_UNDEFINED_TEXT))
        }
    }
    if info.eno == 0 {
        return Err(ErrorBadRequest("<span class=\"small\">少女のひみつを、みだりに暴くものじゃないわ。</span>"));
    }
    // 名前以外の単一項目を取得
    let (name, acronym, color, comment, content, memo): (String, String, [u8; 3], String, String, String) = conn
        .query_row(
            "SELECT c.name,c.acronym,c.color,c.comment,p.content,p.memo FROM character c INNER JOIN character_profile p ON c.eno=?1 AND c.eno=p.eno",
            params![info.eno],
            |row| Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
            ))
        ).map_err(|_| ErrorBadRequest("指定したキャラクターが存在しません"))?;
    // タグを置換
    let replaced_content = common::replace_tag(&content, info.eno, false).map_err(|err| ErrorInternalServerError(err))?;
    let replaced_memo = common::replace_tag(&memo, info.eno, true).map_err(|err| ErrorInternalServerError(err))?;
    // フラグメントを取得
    let mut stmt = conn.prepare("SELECT slot,name,category,lore FROM fragment WHERE eno=?1 ORDER BY slot ASC")
        .map_err(|err| ErrorInternalServerError(err))?;
    let mut fullname: String = String::new();
    let fragments: Vec<_> = stmt.query_map(
        params![info.eno],
        |row| {
            let slot:i8 = row.get(0)?;
            let name: String = row.get(1)?;
            let category: String = row.get(2)?;
            if category == "名前" && fullname == "" {
                fullname = name.clone();
            }
            if slot <= 20 {
                if category != "秘匿" {
                    Ok(Some(ProfileFragment{ name, category, lore: row.get(3)? }))
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        }).map_err(|err| ErrorInternalServerError(err))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| ErrorInternalServerError(err))?
        .iter().flat_map(|x| x.to_owned()).collect();
    // ログインセッションからEnoを取得し、編集モードを有効にするかの確認
    let (edit_mode, name, raw_profile, raw_memo) = if let Some(session) = req.cookie("login_session") {
        // Enoを取得
        let (eno, visit) = common::session_to_eno(&conn, session.value())?;
        if info.eno == eno && visit {
            (true, Some(name), Some(content), Some(memo))
        } else {
            (false, None, None, None)
        }
    } else {
        (false, None, None, None)
    };
    Ok(web::Json::<Profile>(Profile{
        eno: info.eno,
        acronym,
        color,
        comment,
        fullname,
        profile: replaced_content,
        memo: replaced_memo,
        fragments,
        edit_mode,
        name,
        raw_profile,
        raw_memo,
    }))
}

enum SqliteType {
    Text(String),
    Blob(Vec<u8>),
}
#[derive(Deserialize)]
pub(super) struct UpdateProfileData {
    data_type: String,
    value: String,
}
pub(super) async fn update_profile(req: HttpRequest, info: web::Json<UpdateProfileData>) -> Result<String, actix_web::Error> {
    // ログインセッションを取得
    let session =  req.cookie("login_session")
        .ok_or(ErrorBadRequest("ログインセッションがありません"))?;
    // SQL文構築
    let mut sql = "UPDATE ".to_string();
    let value = match info.data_type.as_str() {
        "comment" => {
            if info.value.grapheme_indices(true).count() > 30 {
                return Err(ErrorBadRequest("コメントは30文字以下に設定してください"));
            }
            sql += "character SET comment=?1";
            SqliteType::Text(info.value.clone())
        }
        "acronym" => {
            if info.value.grapheme_indices(true).count() != 1 {
                return Err(ErrorBadRequest("短縮名は1文字である必要があります"));
            }
            sql += "character SET acronym=?1";
            SqliteType::Text(info.value.clone())
        },
        "color" => {
            if info.value.chars().nth(0) != Some('#') {
                return Err(ErrorBadRequest("カラーコードは # から始まる16進数で指定してください"));
            }
            let color = u32::from_str_radix(&info.value[1..], 16).map_err(|err| ErrorBadRequest(err))?;
            sql += "character SET color=?1";
            SqliteType::Blob([(color >> 16) as u8, (color >> 8) as u8, color as u8].to_vec())
        },
        "name" => {
            if info.value.graphemes(true).count() > 16 {
                return Err(ErrorBadRequest("名前は16文字以下に設定してください"));
            }
            sql += "character SET name=?1";
            SqliteType::Text(info.value.clone())
        },
        "profile" => {
            if info.value.graphemes(true).count() > 800 {
                return Err(ErrorBadRequest("プロフィールは800文字以下に設定してください"));
            }
            sql += "character_profile SET content=?1";
            SqliteType::Text(common::html_special_chars(&info.value))
        },
        "memo" => {
            if info.value.graphemes(true).count() > 200 {
                return Err(ErrorBadRequest("メモは200文字以下に設定してください"));
            }
            sql += "character_profile SET memo=?1";
            SqliteType::Text(common::html_special_chars(&info.value))
        },
        "webhook" => {
            sql += "user SET webhook=?1";
            SqliteType::Text(info.value.clone())
        }
        _ => {
            return Err(ErrorBadRequest("更新項目の指定が正しくありません"));
        },
    };
    sql += " WHERE eno=?2";
    let mut params = params![].to_vec();
    match &value {
        SqliteType::Text(value) => params.push(value),
        SqliteType::Blob(value) => params.push(value),
    }
    // データベースに接続
    let conn = common::open_database()?;
    // サーバーの状態を確認
    if let Err(state) = common::check_server_state(&conn)? {
        match state.as_str() {
            "maintenance" => return Err(ErrorServiceUnavailable(common::SERVER_MAINTENANCE_TEXT)),
            "end" => return Err(ErrorServiceUnavailable(common::SERVER_END_TEXT)),
            "littlegirl" => (),
            _ => return Err(ErrorInternalServerError(common::SERVER_UNDEFINED_TEXT)),
        }
    }
    // Enoを取得
    let (eno, visit) = common::session_to_eno(&conn, session.value())?;
    if !visit {
        return Err(ErrorBadRequest("あなたは現在\"Strings\"内にいません"));
    }
    params.push(&eno);
    // データベースを更新
    conn.execute(sql.as_str(), params.as_slice()).map_err(|err| ErrorInternalServerError(err))?;
    match info.data_type.as_str() {
        "profile" => {
            if let SqliteType::Text(v) = value {
                Ok(common::replace_tag(&v, eno, false).map_err(|err| ErrorBadRequest(err))?)
            } else {
                Err(ErrorInternalServerError("なんかおかしなことになっています"))
            }
        }
        "memo" => {
            if let SqliteType::Text(v) = value {
                Ok(common::replace_tag(&v, eno, true).map_err(|err| ErrorBadRequest(err))?)
            } else {
                Err(ErrorInternalServerError("なんかおかしなことになっています"))
            }
        }
        _ => {
            Ok(String::new())
        }
    }
}

#[derive(Deserialize)]
pub(super) struct SendBattleData {
    to: i16,
    plan: u8,
}
pub(super) async fn send_battle(req: HttpRequest, info: web::Json<SendBattleData>) -> Result<String, actix_web::Error> {
    // ログインセッションを取得
    let session =  req.cookie("login_session")
        .ok_or(ErrorBadRequest("ログインセッションがありません"))?;
    // データベースに接続
    let conn = common::open_database()?;
    // サーバーの状態を確認
    if let Err(state) = common::check_server_state(&conn)? {
        match state.as_str() {
            "maintenance" => return Err(ErrorServiceUnavailable(common::SERVER_MAINTENANCE_TEXT)),
            "end" => return Err(ErrorServiceUnavailable(common::SERVER_END_TEXT)),
            "littlegirl" => return Err(ErrorServiceUnavailable(common::SERVER_LITTLEGIRL_TEXT)),
            _ => return Err(ErrorInternalServerError(common::SERVER_UNDEFINED_TEXT))
        }
    }
    // Enoを取得
    let (eno, visit) = common::session_to_eno(&conn, session.value())?;
    if !visit {
        return Err(ErrorBadRequest("あなたは現在\"Strings\"内にいません"));
    }
    // Enoが指定されたものと同じでないのを確認
    if eno == info.to {
        return Err(ErrorBadRequest("自分とは戦闘できません"));
    }
    // 通知を送信
    let handle = actix_rt::spawn(common::send_webhook(info.to, format!("Eno.{} に戦闘を挑まれました。", eno)));
    // 対象の名前を取得
    let target_name: String = conn
    .query_row("SELECT name FROM character WHERE eno=?1", params![info.to], |row| Ok(row.get(0)?))
    .map_err(|err| match err {
        rusqlite::Error::QueryReturnedNoRows => ErrorBadRequest("対象が存在しません"),
        _ => ErrorInternalServerError(err),
    })?;
    // 戦闘予約を送信
    conn.execute("INSERT INTO battle_reserve VALUES(?1,?2,?3)", params![eno, info.to, info.plan])
    .map_err(|err| match err {
        rusqlite::Error::SqliteFailure(err, _) => {
            if err.code == rusqlite::ErrorCode::ConstraintViolation {
                ErrorBadRequest("多分その相手にはもう戦闘を挑んでいます")
            } else {
                ErrorInternalServerError(err)
            }
        },
        _ => ErrorInternalServerError(err),
    })?;
    let _ = handle.await;
    Ok(target_name)
}

#[derive(Deserialize)]
pub(super) struct ReceiveBattleData {
    from: i16,
    plan: u8,
}
pub(super) async fn receive_battle(req: HttpRequest, info: web::Json<ReceiveBattleData>) -> Result<String, actix_web::Error> {
    // ログインセッションを取得
    let session =  req.cookie("login_session")
        .ok_or(ErrorBadRequest("ログインセッションがありません"))?;
    // データベースに接続
    let conn = common::open_database()?;
    // サーバーの状態を確認
    if let Err(state) = common::check_server_state(&conn)? {
        match state.as_str() {
            "maintenance" => return Err(ErrorServiceUnavailable(common::SERVER_MAINTENANCE_TEXT)),
            "end" => return Err(ErrorServiceUnavailable(common::SERVER_END_TEXT)),
            "littlegirl" => return Err(ErrorServiceUnavailable(common::SERVER_LITTLEGIRL_TEXT)),
            _ => return Err(ErrorInternalServerError(common::SERVER_UNDEFINED_TEXT))
        }
    }
    // Enoを取得
    let (eno, visit) = common::session_to_eno(&conn, session.value())?;
    if !visit {
        return Err(ErrorBadRequest("あなたは現在\"Strings\"内にいません"));
    }
    // 戦闘の進行を確認
    let plan: u8 = conn.query_row("SELECT plan FROM battle_reserve WHERE from_eno=?1 AND to_eno=?2", params![info.from, eno], |row| Ok(row.get(0)?))
    .map_err(|err| match err {
        rusqlite::Error::QueryReturnedNoRows => ErrorBadRequest("その相手には戦闘を挑まれていません"),
        _ => ErrorInternalServerError(err),
    })?;
    // 戦闘予告を削除
    conn.execute("DELETE FROM battle_reserve WHERE from_eno=?1 AND to_eno=?2", params![info.from, eno])
        .map_err(|err| ErrorInternalServerError(err))?;
    // プランに合わせて処理分岐
    match &info.plan {
        0 | 1 => {
            if plan == info.plan {
                // 攻撃者に連絡
                let handle = actix_rt::spawn(common::send_webhook(info.from, format!("Eno.{} との戦闘が開始しました。", eno)));
                // 戦闘処理
                let (_, log) = battle::battle([info.from, eno]).map_err(|err| ErrorInternalServerError(err))?;
                let _ = handle.await;
                // 戦闘ログを返す
                Ok(log)
            } else {
                // 戦闘処理を省略して勝敗決定
                // 攻撃者に連絡
                let handle = actix_rt::spawn(common::send_webhook(info.from, format!("Eno.{} との戦闘が省略され、{}しました。", eno, if plan != 0 { "勝利" } else { "敗北" })));
                // フラグメント移動
                let fragment = if plan == 0 {
                    battle::take_fragment(&conn, eno, info.from, &Vec::new())
                        .map_err(|err| ErrorInternalServerError(err))?
                } else {
                    battle::take_fragment(&conn, info.from, eno, &Vec::new())
                        .map_err(|err| ErrorInternalServerError(err))?
                };
                let _ = handle.await;
                // 被攻撃者に連絡
                if let Some(fragment) = fragment {
                    Ok(format!("{{\"result\":\"{}\",\"fragment\":\"{}\"}}", if plan != 0 { "left" } else { "right" }, fragment))
                } else {
                    Ok(format!("{{\"result\":\"{}\"}}", if plan != 0 { "left" } else { "right" }))
                }
            }
        }
        _ => {
            // 攻撃者に連絡
            let handle = actix_rt::spawn(common::send_webhook(info.from, format!("戦闘を挑んでいた Eno.{} に逃走されました。", eno)));
            conn.execute("DELETE FROM battle_reserve WHERE from_eno=?1 AND to_eno=?2", params![info.from, eno])
                .map_err(|err| ErrorInternalServerError(err))?;
            let _ = handle.await;
            // 被攻撃者に連絡
            Ok("{{\"result\":\"escape\"}}".to_string())
        }
    }
}

#[derive(Serialize)]
pub(super) struct BattleReserve {
    from: (i16, String),
    to: (i16, String),
    plan: u8,
}
pub(super) async fn get_battle_reserve(req: HttpRequest) -> Result<web::Json<Vec<BattleReserve>>, actix_web::Error> {
    // ログインセッションを取得
    let session =  req.cookie("login_session")
        .ok_or(ErrorBadRequest("ログインセッションがありません"))?;
    // データベースに接続
    let conn = common::open_database()?;
    // サーバーの状態を確認
    if let Err(state) = common::check_server_state(&conn)? {
        match state.as_str() {
            "maintenance" => return Err(ErrorServiceUnavailable(common::SERVER_MAINTENANCE_TEXT)),
            "end" => (),
            "littlegirl" => (),
            _ => return Err(ErrorInternalServerError(common::SERVER_UNDEFINED_TEXT))
        }
    }
    // Enoを取得
    let (eno, _) = common::session_to_eno(&conn, session.value())?;
    // 取得・整形
    let mut stmt = conn.prepare("WITH r AS (SELECT from_eno,to_eno,plan FROM battle_reserve WHERE ?1 IN (from_eno,to_eno)) SELECT r.from_eno,f.name,r.to_eno,t.name,plan FROM r INNER JOIN character f ON r.from_eno=f.eno INNER JOIN character t ON r.to_eno=t.eno")
        .map_err(|err| ErrorInternalServerError(err))?;
    let result = stmt.query_map(params![eno], |row| {
        Ok(BattleReserve {
            from: (row.get(0)?, row.get(1)?),
            to: (row.get(2)?, row.get(3)?),
            plan: row.get(4)?,
        })
    }).map_err(|err| ErrorInternalServerError(err))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| ErrorInternalServerError(err))?;
    Ok(web::Json(result))
}

#[derive(Deserialize)]
pub(super) struct Eno {
    eno: i16,
}
#[derive(Serialize)]
struct BattleCharacter {
    eno: i16,
    name: String,
    color: [u8; 3]
}
#[derive(Serialize)]
pub(super) struct BattleLog {
    id: i32,
    left: BattleCharacter,
    right: BattleCharacter,
    result: String,
}
pub(super) async fn get_battle_logs(info: web::Query<Eno>) -> Result<web::Json<Vec<BattleLog>>, actix_web::Error> {
    // データベースに接続
    let conn = common::open_database()?;
    // サーバーの状態を確認
    if let Err(state) = common::check_server_state(&conn)? {
        match state.as_str() {
            "maintenance" => return Err(ErrorServiceUnavailable(common::SERVER_MAINTENANCE_TEXT)),
            "end" => (),
            "littlegirl" => (),
            _ => return Err(ErrorInternalServerError(common::SERVER_UNDEFINED_TEXT))
        }
    }
    // 取得・整形
    || -> Result<_, rusqlite::Error> {
        let mut stmt = conn.prepare("WITH b AS (SELECT id,left_eno,right_eno,result FROM battle WHERE ?1 IN (left_eno,right_eno)) SELECT b.id,b.left_eno,l.name,l.color,b.right_eno,r.name,r.color,b.result FROM b INNER JOIN character l ON b.left_eno=l.eno INNER JOIN character r ON b.right_eno=r.eno")?;
        let result = stmt.query_map(params![info.eno], |row| {
            Ok(BattleLog {
                id: row.get(0)?,
                left: BattleCharacter { eno: row.get(1)?, name: row.get(2)?, color: row.get(3)? },
                right: BattleCharacter { eno: row.get(4)?, name: row.get(5)?, color: row.get(6)? },
                result: row.get(7)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        Ok(web::Json(result))
    }().map_err(|err| ErrorInternalServerError(err))
}

#[derive(Deserialize)]
pub(super) struct GetBattleLogData {
    id: i32,
}
pub(super) async fn get_battle_log(info: web::Query<GetBattleLogData>) -> Result<String, actix_web::Error> {
    // データベースに接続
    let conn = common::open_database()?;
    // サーバーの状態を確認
    if let Err(state) = common::check_server_state(&conn)? {
        match state.as_str() {
            "maintenance" => return Err(ErrorServiceUnavailable(common::SERVER_MAINTENANCE_TEXT)),
            "end" => (),
            "littlegirl" => (),
            _ => return Err(ErrorInternalServerError(common::SERVER_UNDEFINED_TEXT))
        }
    }
    // 取得返却
    Ok(
        conn.query_row("SELECT log FROM battle WHERE id=?1", params![info.id], |row| Ok(row.get(0)?))
        .map_err(|err| match err {
            rusqlite::Error::QueryReturnedNoRows => ErrorBadRequest("その戦闘ログは存在しません"),
            _ => ErrorInternalServerError(err),
        })?
    )
}
#[derive(Deserialize)]
pub(super) struct CancelBattleData {
    to: i16,
}
pub(super) async fn cancel_battle(req: HttpRequest, info: web::Json<CancelBattleData>) -> Result<String, actix_web::Error> {
    // ログインセッションを取得
    let session =  req.cookie("login_session")
        .ok_or(ErrorBadRequest("ログインセッションがありません"))?;
    // データベースに接続
    let conn = common::open_database()?;
    // サーバーの状態を確認
    if let Err(state) = common::check_server_state(&conn)? {
        match state.as_str() {
            "maintenance" => return Err(ErrorServiceUnavailable(common::SERVER_MAINTENANCE_TEXT)),
            "end" => return Err(ErrorServiceUnavailable(common::SERVER_END_TEXT)),
            "littlegirl" => (),
            _ => return Err(ErrorInternalServerError(common::SERVER_UNDEFINED_TEXT))
        }
    }
    // Enoを取得
    let (eno, _) = common::session_to_eno(&conn, session.value())?;
    // 通知
    let handle = actix_rt::spawn(common::send_webhook(info.to, format!("Eno.{} に挑まれていた戦闘が取り下げられました。", eno)));
    // 取得・整形
    conn.execute("DELETE FROM battle_reserve WHERE from_eno=?1 AND to_eno=?2", params![eno, info.to])
        .map_err(|err| ErrorBadRequest(err))?;
    let _ = handle.await;
    // 終了
    Ok("戦闘をキャンセルしました".to_string())
}

#[derive(Deserialize)]
pub(super) struct GetLocationData {
    location: Option<String>,
}
#[derive(Serialize)]
pub(super) struct Location {
    name: String,
    lore: Option<String>,
}
pub(super) async fn get_location(req: HttpRequest, info: web::Query<GetLocationData>) -> Result<web::Json<Location>, actix_web::Error> {
    // データベースに接続
    let conn = common::open_database()?;
    // サーバーの状態を確認
    if let Err(state) = common::check_server_state(&conn)? {
        match state.as_str() {
            "maintenance" => return Err(ErrorServiceUnavailable(common::SERVER_MAINTENANCE_TEXT)),
            "end" => (),
            "littlegirl" => (),
            _ => return Err(ErrorInternalServerError(common::SERVER_UNDEFINED_TEXT))
        }
    }
    // ロケーション名を取得
    let name = if let Some(location) = &info.location {
        location.clone()
    } else {
        // ログインセッションを取得
        let session =  req.cookie("login_session")
            .ok_or(ErrorBadRequest("ログインセッションがありません"))?;
        // Enoを取得
        let (eno, _) = common::session_to_eno(&conn, session.value())?;
        // ロケーション名を取得
        conn.query_row("SELECT location FROM character WHERE eno=?1", params![eno], |row| Ok(row.get(0)?))
            .map_err(|err| ErrorInternalServerError(err))?
    };
    // 説明文を取得
    match conn.query_row("SELECT lore FROM location WHERE name=?1", params![name], |row| Ok(row.get(0)?)) {
        Ok(lore) => Ok(web::Json(Location { name: name.to_string(), lore })),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(web::Json(Location { name: name.to_string(), lore: Some("この場所の情報はない。".to_string()) })),
        Err(err) => Err(ErrorInternalServerError(err)),
    }
}

pub(super) async fn delete_character(req: HttpRequest) -> Result<String, actix_web::Error> {
    // ログインセッションを取得
    let session =  req.cookie("login_session")
        .ok_or(ErrorBadRequest("ログインセッションがありません"))?;
    // データベースに接続
    let conn = common::open_database()?;
    // サーバーの状態を確認
    if let Err(state) = common::check_server_state(&conn)? {
        match state.as_str() {
            "maintenance" => return Err(ErrorServiceUnavailable(common::SERVER_MAINTENANCE_TEXT)),
            "end" => return Err(ErrorServiceUnavailable(common::SERVER_END_TEXT)),
            "littlegirl" => (),
            _ => return Err(ErrorInternalServerError(common::SERVER_UNDEFINED_TEXT))
        }
    }
    // Enoを取得
    let (eno, visit) = common::session_to_eno(&conn, session.value())?;
    if !visit {
        return Err(ErrorBadRequest("あなたは現在\"Strings\"内にいません"));
    }
    conn.execute("UPDATE character SET visit=false WHERE eno=?1", params![eno])
        .map_err(|err| ErrorInternalServerError(err))?;
    Ok(String::new())
}

pub(super) async fn teleport_to_master(req: HttpRequest) -> Result<String, actix_web::Error> {
    // ログインセッションを取得
    let session =  req.cookie("login_session")
        .ok_or(ErrorBadRequest("ログインセッションがありません"))?;
    // データベースに接続
    let conn = common::open_database()?;
    // サーバーの状態を確認
    if let Err(state) = common::check_server_state(&conn)? {
        match state.as_str() {
            "maintenance" => return Err(ErrorServiceUnavailable(common::SERVER_MAINTENANCE_TEXT)),
            "end" => return Err(ErrorServiceUnavailable(common::SERVER_END_TEXT)),
            "littlegirl" => (),
            _ => return Err(ErrorInternalServerError(common::SERVER_UNDEFINED_TEXT))
        }
    }
    // Enoを取得
    let (eno, visit) = common::session_to_eno(&conn, session.value())?;
    if !visit {
        return Err(ErrorBadRequest("あなたは現在\"Strings\"内にいません"));
    }
    // 世界観
    let world = conn.query_row("SELECT effect,type FROM skill WHERE id=(SELECT skill FROM fragment WHERE eno=?1 AND slot=1)", params![eno], |row| {
        let timing = battle::Timing::from(row.get::<_, Option<i8>>(1)?.ok_or(rusqlite::Error::InvalidColumnType(1, "skill.type".to_string(), rusqlite::types::Type::Null))?);
        if timing == battle::Timing::World {
            Ok(Some(
                battle::WorldEffect::convert(
                row.get::<_, Option<_>>(0)?
                    .ok_or(rusqlite::Error::InvalidColumnType(0, "skill.effect".to_string(), rusqlite::types::Type::Null))?
                ).map_err(|_| rusqlite::Error::InvalidColumnType(0, "skill.effect".to_string(), rusqlite::types::Type::Blob))?
            ))
        } else {
            Ok(None)
        }
    });
    match world {
        Ok(Some(battle::WorldEffect::森林の従者)) => {
            // ロケーション名を取得
            let master = 154;
            let location: String = conn.query_row("SELECT location FROM character WHERE eno=?1", params![master], |row| Ok(row.get(0)?))
                .map_err(|err| ErrorInternalServerError(err))?;
            // 移動
            conn.execute("UPDATE character SET location=?2 WHERE eno=?1", params![eno, location])
                .map_err(|err| ErrorInternalServerError(err))?;
            Ok(format!("{}に移動しました", location))
        }
        Ok(_) | Err(rusqlite::Error::QueryReturnedNoRows) => {
            Err(ErrorBadRequest("世界観の効果がありません"))
        }
        Err(err) => Err(ErrorInternalServerError(err))
    }
}