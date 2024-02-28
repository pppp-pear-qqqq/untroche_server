use std::{collections::hash_map::DefaultHasher, hash::{Hash, Hasher}, i128};

use actix_rt;
use actix_web::{error::{ErrorBadRequest, ErrorInternalServerError}, HttpRequest, web};
use rusqlite::{named_params, params, types::Type};
use serde::{Deserialize, Serialize};
use unicode_segmentation::UnicodeSegmentation;
use uuid::Uuid;

use super::{battle, common, FormFragment};

// アプリケーション部分
#[derive(Deserialize)]
pub(super) struct LoginData {
    eno: i16,
    password: String,
}
pub(super) async fn login(info: web::Json<LoginData>) -> Result<String, actix_web::Error> {
    // データベースに接続
    let conn = common::open_database()?;
    // postで受け取ったパスワードをハッシュ化する
    let mut hasher = DefaultHasher::new();
    info.password.hash(&mut hasher);
    // データベースを探索
    let eno = conn.query_row(
        "SELECT eno FROM user WHERE eno=?1 AND password=?2",
        params![info.eno, hasher.finish() as i64],
        |row| row.get::<usize, i16>(0),
    ).map_err(|err| ErrorBadRequest(err))?;
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
    if info.name.grapheme_indices(true).count() > 16 {
        return Err(ErrorBadRequest("名前が長すぎます"));
    } else if info.acronym.grapheme_indices(true).count() != 1 {
        return Err(ErrorBadRequest("短縮名が1文字ではありません"));
    } else if info.fragment.len() > 5 {
        return Err(ErrorBadRequest("取得しようとしているフラグメントが多すぎます"));
    }
    println!("{}", &info.color[1..]);
    let color_raw = u32::from_str_radix(&info.color[1..], 16).map_err(|err| ErrorBadRequest(err))?;
    let color = [(color_raw >> 16) as u8, (color_raw >> 8) as u8, color_raw as u8];
    // データベースに接続
    let conn = common::open_database()?;
    // 発生するエラーはすべてInternalServerError相当なので、クロージャに格納してまとめてmapしている
    || -> Result<_, rusqlite::Error> {
        // 受け取ったパスワードをハッシュ化
        let mut hasher = DefaultHasher::new();
        info.password.hash(&mut hasher);
        // データベースに新規ユーザーを追加
        conn.execute("INSERT INTO user(password) VALUES(?1)", params![hasher.finish() as i64])?;
        let eno = conn.last_insert_rowid() as i16;
        conn.execute("INSERT INTO character VALUES(?1,?2,?3,?4,?5,?6,?7,?8)", params![eno, info.name, info.acronym, color, "", "門", 0, "{}"])?;
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
    let mut word = common::html_special_chars(info.word.clone());
    // 入力情報が正しい長さであることを確認
    if info.name.graphemes(true).count() <= 20 && word.graphemes(true).count() <= 600 {
		// データベースに接続
		let conn = common::open_database()?;
        // ログインセッションをデータベースと照会
        let eno = common::session_to_eno(&conn, session.value())?;
        // 発言色・現在地を取得
        let (color, location): ([u8; 3], String) = conn.query_row("SELECT color,location FROM character WHERE eno=?1", params![eno], |row|Ok((row.get(0)?, row.get(1)?)))
            .map_err(|err| ErrorInternalServerError(err))?;
        // タグを置換
        word = common::replace_tag(word, eno, false).map_err(|err| ErrorInternalServerError(err))?;
        // 発言をデータベースに格納
        conn.execute(
            "INSERT INTO timeline(from_eno,to_eno,location,color,name,word) VALUES(?1,?2,?3,?4,?5,?6)",
            params![eno, info.to, if info.location { None } else { Some(location) }, color, info.name, word],
        ).map_err(|err| ErrorInternalServerError(err))?;
        if let Some(to) = &info.to {
            println!("Webhook送信");
            let _ = actix_rt::spawn(common::send_webhook(*to, format!("Eno.{} {}からの発言を受けました。", to, info.name))).await;
        }
        Ok("発言に成功しました".to_string())
    } else {
        Err(ErrorBadRequest("名前、または発言内容が長すぎます"))
    }
}

#[derive(Deserialize)]
pub(super) struct GetChatData {
    num: i32,                   // 取得件数
    start: Option<i32>,         // 取得開始位置
    from: Option<i16>,          // 発言者
    to: Option<i16>,            // 対象者
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
    name: String,
    word: String,
}
pub(super) async fn get_chat(req: HttpRequest, info: web::Query<GetChatData>) -> Result<web::Json<Vec<Chat>>, actix_web::Error> {
    // データベースに接続
    let conn = common::open_database()?;
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
            if let Some(_) = eno {
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
        sql.push("id<:start");
        params.push((":start", start));
    }
    // 発言者
    if let Some(from) = &info.from {
        sql.push("from_eno=:from");
        params.push((":from", if *from == 0 { &eno } else { from }));
    }
    // 対象者
    if let Some(to) = &info.to {
        sql.push("to_eno=:to");
        params.push((":to", if *to == 0 { &eno } else { to }));
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
    let sql = sql.join(" AND ");
    let sql = "SELECT id,timestamp,from_eno,to_eno,location,color,name,word FROM timeline".to_string() + if sql != "" { " WHERE " } else { "" } + &sql + " ORDER BY id DESC LIMIT :num";
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
                    color: row.get(5)?,
                    name: row.get(6)?,
                    word: row.get(7)?,
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
            if let Some(_) = eno {
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
    let sql = "SELECT eno,name,acronym,color,comment,location FROM character".to_string() + if sql != "" { " WHERE " } else { "" } + &sql + " ORDER BY eno ASC LIMIT :num";
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
    name: String,
    word: String,
    default_name: String,
    lore: String,
    timing: battle::Timing,
    effect: Vec<battle::Command>,
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
}
pub(super) async fn get_fragments(req: HttpRequest) -> Result<web::Json<Vec<Fragment>>, actix_web::Error> {
    // ログインセッションを取得
    let session =  req.cookie("login_session")
        .ok_or(ErrorBadRequest("ログインセッションがありません"))?;
    // データベースに接続
    let conn = common::open_database()?;
    // Enoを取得
    let eno = common::session_to_eno(&conn, session.value())?;
    // 取得・整形
    || -> Result<_, rusqlite::Error> {
        let mut stmt = conn.prepare("WITH f AS (SELECT slot,category,name,lore,status,skill,skillname,skillword FROM fragment WHERE eno=?1) SELECT f.slot,f.category,f.name,f.lore,f.status,f.skillname,f.skillword,s.name,s.lore,s.type,s.effect FROM f LEFT OUTER JOIN skill s ON f.skill=s.id")?;
        let result = stmt.query_map(params![eno], |row| {
            // ステータスの原型
            let status: Vec<u8> = row.get(4)?;
            // スキル効果の原型
            let effect:Option<Vec<u8>> = row.get(10)?;
            // スキルの有無を判定・整形
            let skill = if let Some(effect) = effect {
                Some(Skill { 
                    name: row.get::<usize, Option<_>>(5)?.unwrap_or(String::new()),
                    word: row.get::<usize, Option<_>>(6)?.unwrap_or(String::new()),
                    default_name: row.get::<usize, Option<_>>(7)?.ok_or(rusqlite::Error::InvalidColumnType(0, "skill.lore".to_string(), Type::Text))?,
                    lore: row.get::<usize, Option<_>>(8)?.ok_or(rusqlite::Error::InvalidColumnType(0, "skill.lore".to_string(), Type::Text))?,
                    timing: battle::Timing::from(row.get::<usize, Option<_>>(9)?.ok_or(rusqlite::Error::InvalidColumnType(0, "skill.type".to_string(), Type::Integer))?),
                    effect: battle::Command::convert(effect).map_err(|_| rusqlite::Error::InvalidColumnType(0, "skill.effect".to_string(), Type::Text))?,
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
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        Ok(web::Json(result))
    }().map_err(|err| ErrorInternalServerError(err))
}

#[derive(Deserialize)]
struct MoveFragmentData {
    prev: i8,
    next: i8,
    skill_name: Option<String>,
    skill_word: Option<String>,
}
#[derive(Deserialize)]
pub(super) struct UpdateFragmentsData {
    change: Vec<MoveFragmentData>,
    trash: Vec<i8>,
}
struct UpdateFragment {
    slot: i8,
    category: String,
    name: String,
    lore: String,
    status: [u8; 8],
    skill: Option<i32>,
}
pub(super) async fn update_fragments(req: HttpRequest, info: web::Json<UpdateFragmentsData>) -> Result<String, actix_web::Error> {
    // ログインセッションを取得
    let session =  req.cookie("login_session")
        .ok_or(ErrorBadRequest("ログインセッションがありません"))?;
    // データベースに接続
    let conn = common::open_database()?;
    // Enoを取得
    let eno = common::session_to_eno(&conn, session.value())?;
    // フラグメントの一覧を取得
    // SQLの時点で精査するのが面倒なので、とりあえず全部取得する
    let result = || -> Result<_, rusqlite::Error> {
        let mut stmt = conn.prepare("SELECT slot,category,name,lore,status,skill FROM fragment WHERE eno=?1")?;
        let result = stmt.query_map(params![eno], |row| {
            Ok(UpdateFragment {
                slot: row.get(0)?,
                category: row.get(1)?,
                name: row.get(2)?,
                lore: row.get(3)?,
                status: row.get(4)?,
                skill: row.get(5)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;
        Ok(result)
    }().map_err(|err| ErrorInternalServerError(err))?;
    // 廃棄するものの得点付け、および一旦削除するもののリストアップ
    let mut kins = 0;
    let mut delete_list: Vec<&i8> = Vec::new();
    for x in &result {
        if info.trash.contains(&x.slot) {
            kins += common::calc_fragment_kins(x.status);
            delete_list.push(&x.slot);
        } else if info.change.iter().any(|y| y.prev == x.slot) {
            delete_list.push(&x.slot);
        }
    }
    // 削除のためのSQL文作成
    let mut sql = "DELETE FROM fragment WHERE eno=?1 AND slot IN (".to_string();
    let mut params = params![eno].to_vec();
    let mut i = 2i8;
    for v in delete_list {
        sql += &format!("{}?{}", if i > 2 {","} else {""}, i);
        i += 1;
        params.push(v);
    }
    sql += ")";
    // 削除
    conn.execute(&sql, params.as_slice()).map_err(|err| ErrorInternalServerError(err))?;
    // キンス反映
    if kins != 0 {
        conn.execute("UPDATE character SET kins=kins+?1 WHERE eno=?2", params![kins, eno])
            .map_err(|err| ErrorInternalServerError(err))?;
    }
    // 更新反映
    // SQL文とバインドの対応を構築するのが本当に難しかったので、一旦ひとつずつINSERTしていく形にする
    for x in &info.change {
        if let Some(y) = result.iter().find(|y| y.slot == x.prev) {
            conn.execute(
                "INSERT INTO fragment VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
                params![
                    eno,
                    x.next,
                    y.category,
                    y.name,
                    y.lore,
                    y.status,
                    y.skill,
                    x.skill_name,
                    x.skill_word,
                ],
            ).map_err(|err| ErrorInternalServerError(err))?;
        } else {
            return Err(ErrorInternalServerError("存在しないフラグメントを変更しようとしました"));
        }
    }
    Ok("フラグメントを更新しました".to_string())
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
    let replaced_content = common::replace_tag(content.clone(), info.eno, false).map_err(|err| ErrorInternalServerError(err))?;
    let replaced_memo = common::replace_tag(memo.clone(), info.eno, true).map_err(|err| ErrorInternalServerError(err))?;
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
                Ok(Some(ProfileFragment{ name, category, lore: row.get(3)? }))
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
        let eno = common::session_to_eno(&conn, session.value())?;
        if info.eno == eno {
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
            SqliteType::Text(common::html_special_chars(info.value.clone()))
        },
        "memo" => {
            if info.value.graphemes(true).count() > 200 {
                return Err(ErrorBadRequest("メモは200文字以下に設定してください"));
            }
            sql += "character_profile SET memo=?1";
            SqliteType::Text(common::html_special_chars(info.value.clone()))
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
    // Enoを取得
    let eno = common::session_to_eno(&conn, session.value())?;
    params.push(&eno);
    // データベースを更新
    conn.execute(sql.as_str(), params.as_slice()).map_err(|err| ErrorInternalServerError(err))?;
    match info.data_type.as_str() {
        "profile" => {
            if let SqliteType::Text(v) = value {
                Ok(common::replace_tag(v, eno, false).map_err(|err| ErrorBadRequest(err))?)
            } else {
                Err(ErrorInternalServerError("なんかおかしなことになっています"))
            }
        }
        "memo" => {
            if let SqliteType::Text(v) = value {
                Ok(common::replace_tag(v, eno, true).map_err(|err| ErrorBadRequest(err))?)
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
    // Enoを取得
    let eno = common::session_to_eno(&conn, session.value())?;
    // Enoが指定されたものと同じでないのを確認
    if eno == info.to {
        return Err(ErrorBadRequest("自分とは戦闘できません"));
    }
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
    // 通知を送信
    actix_web::rt::spawn(common::send_webhook(info.to, format!("Eno.{} に戦闘を挑まれました。", eno)));
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
    // Enoを取得
    let eno = common::session_to_eno(&conn, session.value())?;
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
                // 戦闘処理
                let log = battle::battle(info.from, eno).map_err(|err| ErrorInternalServerError(err))?;
                let log_json = serde_json::to_string(&log).map_err(|err| ErrorInternalServerError(err))?;
                // データベースに保存
                conn.execute(
                    "INSERT INTO battle(left_eno,right_eno,result,log) VALUES(?1,?2,?3,?4)",
                    params![
                        info.from,
                        eno,
                        log.result,
                        log_json,
                    ]
                ).map_err(|err| ErrorInternalServerError(err))?;
                // フラグメント移動
                if let Some((win, lose)) = match log.result.as_str() {
                    "left" => Some((info.from, eno)),
                    "right" => Some((eno, info.from)),
                    _ => None,
                } {
                    let mut stmt = conn.prepare("SELECT slot,category,name,lore,status,skill FROM fragment WHERE eno=?1")
                        .map_err(|err| ErrorInternalServerError(err))?;
                    // 候補の取得
                    let result = stmt.query_map(params![lose], |row| {
                        Ok(UpdateFragment{
                            slot: row.get(0)?,
                            category: row.get(1)?,
                            name: row.get(2)?,
                            lore: row.get(3)?,
                            status: row.get(4)?,
                            skill: row.get(5)?,
                        })
                    }).map_err(|err| ErrorInternalServerError(err))?
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(|err| ErrorInternalServerError(err))?;
                    // 移動対象を決定
                    let t = result.get(rand::random::<usize>() % result.len())
                        .ok_or(ErrorInternalServerError("フラグメントの移動に失敗しました"))?;
                    // 勝利者にフラグメントを追加
                    common::add_fragment(&conn, win, &common::Fragment::new(t.category.to_owned(), t.name.to_owned(), t.lore.to_owned(), t.status, t.skill))
                        .map_err(|err| ErrorInternalServerError(err))?;
                    // フラグメントの削除
                    conn.execute("DELETE FROM fragment WHERE eno=?1 AND slot=?2", params![lose, t.slot])
                        .map_err(|err| ErrorInternalServerError(err))?;
                }
                // 攻撃者に連絡
                actix_web::rt::spawn(common::send_webhook(info.from, format!("Eno.{} との戦闘が開始しました。", eno)));
                // 戦闘ログを返す
                Ok(log_json)
            } else {
                // 戦闘処理を省略して勝敗決定
                // 攻撃者に連絡
                actix_web::rt::spawn(common::send_webhook(info.from, format!("Eno.{} との戦闘が省略され、{}しました。", eno, if plan != 0 { "勝利" } else { "敗北" })));
                // 被攻撃者に連絡
                Ok(format!("{{\"result\":\"omission\",\"content\":\"Eno.{} との戦闘が省略され、{}しました\"}}", info.from, if info.plan != 0 { "勝利" } else { "敗北" }))
            }
        }
        _ => {
            conn.execute("DELETE FROM battle_reserve WHERE from_eno=?1 AND to_eno=?2", params![info.from, eno])
                .map_err(|err| ErrorInternalServerError(err))?;
            // 攻撃者に連絡
            actix_web::rt::spawn(common::send_webhook(info.from, format!("戦闘を挑んでいた Eno.{} に逃走されました。", eno)));
            // 被攻撃者に連絡
            Ok(format!("{{\"result\":\"omission\",\"content\":\"Eno.{} との戦闘から逃走しました\"}}", info.from))
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
    // Enoを取得
    let eno = common::session_to_eno(&conn, session.value())?;
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
    // Enoを取得
    let eno = common::session_to_eno(&conn, session.value())?;
    // 取得・整形
    conn.execute("DELETE FROM battle_reserve WHERE from_eno=?1 AND to_eno=?2", params![eno, info.to])
        .map_err(|err| ErrorBadRequest(err))?;
    // 通知
    actix_web::rt::spawn(common::send_webhook(info.to, format!("Eno.{} に挑まれていた戦闘が取り下げられました。", eno)));
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
    lore: String,
}
pub(super) async fn get_location(req: HttpRequest, info: web::Query<GetLocationData>) -> Result<web::Json<Location>, actix_web::Error> {
    // データベースに接続
    let conn = common::open_database()?;
    if let Some(location) = &info.location {
        // 説明文を取得
        let lore: String = conn.query_row("SELECT lore FROM location WHERE name=?1", params![location], |row| Ok(row.get(0)?))
            .map_err(|err| match err {
                rusqlite::Error::QueryReturnedNoRows => ErrorBadRequest("この場所の情報はありません"),
                _ => ErrorInternalServerError(err),
            })?;
        Ok(web::Json(Location { name: location.to_string(), lore }))
    } else {
        // ログインセッションを取得
        let session =  req.cookie("login_session")
            .ok_or(ErrorBadRequest("ログインセッションがありません"))?;
        // Enoを取得
        let eno = common::session_to_eno(&conn, session.value())?;
        // 取得・返却
        Ok(web::Json(
            conn.query_row(
                "SELECT name,lore FROM location WHERE name=(SELECT location FROM character WHERE eno=?1)",
                params![eno],
                |row| Ok(Location {
                    name: row.get(0)?,
                    lore: row.get(1)?,
                })
            ).map_err(|err| match err {
                rusqlite::Error::QueryReturnedNoRows => ErrorBadRequest("この場所の情報はありません"),
                _ => ErrorInternalServerError(err),
            })?
        ))
    }
}